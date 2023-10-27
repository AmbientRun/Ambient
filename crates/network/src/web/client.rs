use ambient_app::{world_instance_resources, AppResources};
use ambient_core::{asset_cache, gpu, RuntimeKey};
use ambient_ecs::{world_events, Entity, SystemGroup};
use ambient_element::{
    consume_context, provide_context, use_frame, use_local_task, use_state, use_state_with,
    Element, ElementComponent, ElementComponentExt, Hooks,
};
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    download_asset::ReqwestClientKey,
    Cb,
};
use ambient_renderer::RenderTarget;
use ambient_rpc::RpcRegistry;
use ambient_sys::{task::RuntimeHandle, time::Instant};
use ambient_ui_native::{Centered, Dock, FlowColumn, FlowRow, StylesExt, Text, Throbber};
use anyhow::Context;
use bytes::{BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use glam::uvec2;
use parking_lot::Mutex;
use std::{sync::Arc, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::Url;

use crate::{
    client::{CleanupFunc, ClientState, Control, GameClientRenderTarget, LoadedFunc},
    client_game_state::{game_screen_render_target, ClientGameState},
    log_network_result,
    proto::{
        client::{ClientProtoState, SharedClientGameState},
        ClientRequest,
    },
    server::RpcArgs,
    stream::{FramedRecvStream, FramedSendStream, RawFramedRecvStream},
    web::WebTransportProxy,
    webtransport::Connection,
    NetworkError,
};

use super::ProxyMessage;

const ALLOWED_FRAME_TIME: Duration = Duration::from_nanos(16_666_666); // 1/60 s
const MAX_ACCUMMULATED_FRAME_DELAY: Duration = Duration::from_millis(50);

#[derive(Debug, Clone)]
pub struct GameClientView {
    /// The url to connect to
    pub url: String,
    pub user_id: String,
    pub fail_on_version_mismatch: bool,
    pub systems_and_resources: Cb<dyn Fn() -> (SystemGroup, Entity) + Sync + Send>,
    /// Invoked when the game client is loaded
    ///
    /// The returned function is executed when the client is disconnected
    pub on_loaded: LoadedFunc,
    pub create_rpc_registry: Cb<dyn Fn() -> RpcRegistry<RpcArgs> + Sync + Send>,
    pub inner: Element,
}

// Dock(vec![Text::el("Error").header_style(), Text::el(error)]).el()
impl ElementComponent for GameClientView {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self {
            url,
            user_id,
            fail_on_version_mismatch,
            systems_and_resources,
            on_loaded,
            create_rpc_registry,
            inner,
        } = *self;

        let assets = hooks.world.resource(asset_cache()).clone();

        let gpu = hooks.world.resource(gpu()).clone();

        provide_context(hooks, || {
            GameClientRenderTarget(Arc::new(RenderTarget::new(&gpu, uvec2(1, 1), None)))
        });

        let (render_target, _) = consume_context::<GameClientRenderTarget>(hooks).unwrap();

        // The game client will be set once a connection establishes
        let (client_state, set_client_state) = use_state(hooks, None as Option<ClientState>);

        // When the client is connected, run the update logic each frame
        if let Some(client_state) = &client_state {
            run_game_logic(
                hooks,
                client_state.game_state.clone(),
                render_target.clone(),
            );
        }

        // TODO: allow remote shutdown
        let ((_control_tx, control_rx), _) = use_state_with(hooks, |_| flume::unbounded());

        let (err, set_error) = use_state(hooks, None);

        use_local_task(hooks, move |ui_world| {
            let local_resources = world_instance_resources(AppResources::from_world(ui_world))
                .with(game_screen_render_target(), render_target.0.clone());

            let task = async move {
                let mut url = Url::parse(&url).context("Malformed Url")?;

                if url.path() == "/servers/ensure-running" {
                    url = resolve_hosted_server(&assets, url).await?;
                }

                let conn = Connection::connect(&url.as_str()).await.with_context(|| {
                    format!("Failed to establish a WebTransport session for \"{url}\"")
                })?;

                tracing::debug!("Established WebTransport session");

                let (proxy_tx, proxy_rx) = flume::bounded(32);

                let mut proxy_tx = Some(WebTransportProxy::new(proxy_tx));
                // Create a handle for the game client

                handle_connection(
                    conn,
                    &assets,
                    user_id,
                    fail_on_version_mismatch,
                    move |assets, user_id| {
                        let (systems, resources) = systems_and_resources();

                        let resources = local_resources
                            .clone()
                            .with(ambient_core::player::local_user_id(), user_id.into())
                            .with_merge(resources);

                        let game_state = ClientGameState::new(
                            &gpu,
                            assets.clone(),
                            user_id.into(),
                            systems,
                            resources,
                        );

                        // Create a handle for the game client
                        let client_state = ClientState::new(
                            Arc::new(proxy_tx.take().expect("on_loaded called twice")),
                            Arc::new(create_rpc_registry()),
                            Arc::new(Mutex::new(game_state)),
                            user_id.into(),
                        );

                        let game_state = &client_state.game_state;
                        let cleanup = {
                            // Lock before setting
                            let game_state = &mut game_state.lock();

                            // Updates the game client context in the Ui tree
                            // Update the resources on the client side world to reflect the new connection
                            // state

                            game_state.world.add_resource(
                                crate::client::client_state(),
                                Some(client_state.clone()),
                            );

                            (on_loaded)(&client_state, game_state)?
                        };

                        // Set the client last so that the game state is initialized first
                        set_client_state(Some(client_state.clone()));

                        Ok((game_state.clone(), cleanup))
                    },
                    control_rx,
                    proxy_rx,
                )
                .await?;

                Ok(()) as anyhow::Result<()>
            };

            async move {
                match task.await {
                    Ok(()) => {
                        tracing::info!("Client disconnected");
                    }
                    Err(err) => {
                        if let Some(err) = err.downcast_ref::<NetworkError>() {
                            if let NetworkError::ConnectionClosed = err {
                                tracing::info!("Connection closed by peer");
                            } else {
                                tracing::error!("Network error: {:?}", err);
                            }
                        } else {
                            tracing::error!("Game failed: {:?}", err);
                        }
                        set_error(Some(format!("{err:?}")));
                    }
                }
            }
        });

        if let Some(err) = err {
            return Dock(vec![Text::el("Error").header_style(), Text::el(err)]).el();
        }

        if let Some(client_state) = client_state {
            // Provide the context
            provide_context(hooks, || client_state.clone());
            hooks
                .world
                .add_resource(crate::client::client_state(), Some(client_state.clone()));

            inner
        } else {
            Centered(vec![FlowColumn::el([FlowRow::el([
                Text::el("Connecting"),
                Throbber.el(),
            ])])])
            .el()
        }
    }
}

#[derive(Debug)]
struct FrameDropStats {
    frames: usize,
    dropped: usize,
    last_warning: Instant,
    warn_freq: usize,
}
impl FrameDropStats {
    pub fn new(warn_freq: usize) -> Self {
        Self {
            frames: 0,
            dropped: 0,
            last_warning: Instant::now(),
            warn_freq,
        }
    }

    pub fn on_frame(&mut self) {
        self.frames += 1;
    }

    pub fn on_dropped_frame(&mut self, now: Instant) {
        self.dropped += 1;

        if self.dropped == self.warn_freq {
            let dropped_time = now - self.last_warning;
            tracing::warn!(
                "Too much accummulated frame delay! Dropped {:.2}% of frames in last {:?}",
                self.drop_ratio() * 100.0,
                dropped_time
            );
            self.frames = 0;
            self.dropped = 0;
            self.last_warning = now;
        }
    }

    fn drop_ratio(&self) -> f32 {
        self.dropped as f32 / self.frames as f32
    }
}

/// Decides if a frame should be dropped based on observed frame times.
///
/// Frames taking too much time can starve browser networking (https://github.com/w3c/webtransport/issues/543)
/// We calculate how much delay has been accummulated and when a threshold is reached we drop a frame to allow
/// for networking to catch up.
struct FrameDropping {
    stats: FrameDropStats,
    accummulated_delay: Duration,
    last_frame_timestamp: Option<Instant>,
}
impl FrameDropping {
    pub fn new() -> Self {
        Self {
            stats: FrameDropStats::new(120),
            accummulated_delay: Duration::ZERO,
            last_frame_timestamp: None,
        }
    }

    pub fn should_drop(&mut self, now: Instant) -> bool {
        self.stats.on_frame();

        if let Some(last) = self.last_frame_timestamp {
            self.observe_frame_time(now - last);
        }
        self.last_frame_timestamp = Some(now);

        if self.accummulated_delay > MAX_ACCUMMULATED_FRAME_DELAY {
            self.accummulated_delay = Duration::ZERO;
            self.stats.on_dropped_frame(now);
            true
        } else {
            false
        }
    }

    fn observe_frame_time(&mut self, frame_time: Duration) {
        self.accummulated_delay = if frame_time < ALLOWED_FRAME_TIME {
            // frame was processed in time -> reset the accummulated delay
            Duration::ZERO
        } else {
            self.accummulated_delay + frame_time
        };
    }
}

fn run_game_logic(
    hooks: &mut Hooks,
    game_state: SharedClientGameState,
    render_target: GameClientRenderTarget,
) {
    let world_event_reader = Mutex::new(hooks.world.resource(world_events()).reader());

    let gpu = hooks.world.resource(gpu()).clone();

    #[cfg(feature = "frame-dropping")]
    let frame_dropping = Mutex::new(FrameDropping::new());

    use_frame(hooks, move |app_world| {
        #[cfg(feature = "frame-dropping")]
        {
            let now = Instant::now();
            let mut frame_dropping = frame_dropping.lock();
            if frame_dropping.should_drop(now) {
                return;
            }
        }

        let mut game_state = game_state.lock();

        // Pipe events from app world to game world
        for (_, event) in world_event_reader
            .lock()
            .iter(app_world.resource(world_events()))
        {
            game_state
                .world
                .resource_mut(world_events())
                .add_event(event.clone());
        }

        game_state.on_frame(&gpu, &render_target.0);
    });
}

async fn handle_connection(
    mut conn: Connection,
    assets: &AssetCache,
    user_id: String,
    fail_on_version_mismatch: bool,
    mut on_loaded: impl FnMut(&AssetCache, &str) -> anyhow::Result<(SharedClientGameState, CleanupFunc)>,
    control_rx: flume::Receiver<Control>,
    proxy_rx: flume::Receiver<ProxyMessage>,
) -> anyhow::Result<()> {
    let runtime = RuntimeKey.get(&assets);

    let mut request_send = FramedSendStream::new(conn.open_uni().await?);

    // Accept the diff and stat stream
    // Nothing is read from them until the connection has been accepted

    // Send a connection request
    tracing::debug!("Attempting to connect using {user_id:?}");

    request_send
        .send(ClientRequest::Connect(user_id.clone()))
        .await?;

    let mut client = ClientProtoState::Pending(user_id.clone());

    let mut push_recv = FramedRecvStream::new(
        conn.accept_uni()
            .await
            .ok_or(NetworkError::ConnectionClosed)??,
    );

    while client.is_pending() {
        tracing::info!("Waiting for server to accept connection and send server info");
        if let Some(frame) = push_recv.next().await {
            client.process_push(&assets, fail_on_version_mismatch, frame?)?;
        }
    }

    if !client.is_connected() {
        tracing::warn!("Connection failed or was denied");
        return Ok(());
    }

    let mut diff_stream = RawFramedRecvStream::new(
        conn.accept_uni()
            .await
            .ok_or(NetworkError::ConnectionClosed)??,
    );

    let (shared_client_state, cleanup) = on_loaded(&assets, &user_id)?;
    let on_disconnect = move || cleanup();

    scopeguard::defer!(on_disconnect());

    let mut control_rx = control_rx.into_stream();
    let mut proxy_rx = proxy_rx.into_stream();

    tracing::info!("Client connected");

    while let ClientProtoState::Connected(connected) = &mut client {
        tokio::select! {
            Some(frame) = push_recv.next() => {
                client.process_push(&assets, fail_on_version_mismatch, frame?)?;
            }

            Some(message) = proxy_rx.next() => {
                handle_request(&mut conn, &runtime, message).await?;
            }

            Some(control) = control_rx.next() => {
                match control {
                    Control::Disconnect => {
                        tracing::info!("Disconnecting manually");
                        // Tell the server that we want to gracefully disconnect
                        request_send.send(ClientRequest::Disconnect).await?;
                    }
                }
            }

            Some(Ok(datagram)) = conn.read_datagram() => {
                connected.process_datagram(&shared_client_state, datagram)?;
            }

            Some(Ok((send, recv))) = conn.accept_bi() => {
                connected.process_bi(&shared_client_state, send, recv);
            }

            Some(Ok(recv)) = conn.accept_uni() => {
                connected.process_uni(&shared_client_state, recv);
            }

            Some(diff) = diff_stream.next() => {
                connected.process_diff(&shared_client_state, diff?)?;
            }
        }
    }

    tracing::debug!("Client entered disconnected state");
    Ok(())
}

/// Handles a request from the Send+Sync proxy object
async fn handle_request(
    conn: &mut Connection,
    runtime: &RuntimeHandle,
    message: ProxyMessage,
) -> Result<(), NetworkError> {
    match message {
        ProxyMessage::RequestBi { id, mut data, resp } => {
            let (mut send, mut recv) = conn.open_bi().await?;

            runtime.spawn_local(async move {
                log_network_result!(
                    async {
                        send.write_u32(id).await?;
                        send.write_all_buf(&mut data).await?;
                        Ok(()) as Result<(), NetworkError>
                    }
                    .await
                )
            });

            runtime.spawn_local(async move {
                log_network_result!(
                    async {
                        let mut buf = Vec::new();
                        recv.read_to_end(&mut buf).await?;

                        resp.send(buf.into()).ok();
                        Ok(()) as Result<(), NetworkError>
                    }
                    .await
                )
            });

            Ok(())
        }
        ProxyMessage::RequestUni { id, mut data } => {
            let mut send = conn.open_uni().await?;

            runtime.spawn_local(async move {
                log_network_result!(
                    async {
                        send.write_u32(id).await?;
                        send.write_all_buf(&mut data).await?;

                        Ok(()) as Result<(), NetworkError>
                    }
                    .await
                )
            });

            Ok(())
        }
        ProxyMessage::Datagram { id, data } => {
            let mut bytes = BytesMut::with_capacity(4 + data.len());

            bytes.put_u32(id);
            bytes.put_slice(&data);

            let fut = conn.send_datagram(&bytes[..]);
            runtime.spawn_local(async move {
                log_network_result!(fut.await);
            });

            Ok(())
        }
    }
}

async fn resolve_hosted_server(assets: &AssetCache, url: Url) -> anyhow::Result<Url> {
    tracing::debug!("Resolving hosted server at {url}");

    let client = ReqwestClientKey.get(assets);

    let res = client
        .get(url.clone())
        .send()
        .await
        .context("Failed to resolve hosted server")?
        .text()
        .await
        .context("Failed to get result for request")?;

    Url::parse(&format!("https://{}", res.trim())).with_context(|| {
        format!("Expected a valid Url resolving host, but was unable to resolve {res}")
    })
}
