use ambient_app::{world_instance_resources, AppResources};
use ambient_core::{asset_cache, gpu, RuntimeKey};
use ambient_ecs::{world_events, Entity, SystemGroup};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_renderer::RenderTarget;
use ambient_rpc::RpcRegistry;
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    Cb,
};
use ambient_sys::{task::RuntimeHandle, time::sleep};
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
    stream::{FramedRecvStream, FramedSendStream},
    web::WebTransportProxy,
    webtransport::Connection,
    NetworkError,
};

use super::ProxyMessage;

#[derive(Debug, Clone)]
pub struct GameClientView {
    /// The url to connect to
    pub url: Url,
    pub user_id: String,
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
            systems_and_resources,
            on_loaded,
            create_rpc_registry,
            inner,
        } = *self;

        let gpu = hooks.world.resource(gpu()).clone();

        hooks.provide_context(|| {
            GameClientRenderTarget(Arc::new(RenderTarget::new(&gpu, uvec2(1, 1), None)))
        });

        let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();

        let assets = hooks.world.resource(asset_cache()).clone();

        // The game client will be set once a connection establishes
        let (client_state, set_client_state) = hooks.use_state(None as Option<ClientState>);

        // When the client is connected, run the update logic each frame
        if let Some(client_state) = &client_state {
            tracing::info!("Adding game logic hook");
            run_game_logic(
                hooks,
                client_state.game_state.clone(),
                render_target.clone(),
            );
        }

        // TODO: allow remote shutdown
        let ((_control_tx, control_rx), _) = hooks.use_state_with(|_| flume::unbounded());

        let (err, set_error) = hooks.use_state(None);

        hooks.use_local_task(move |ui_world| {
            let local_resources = world_instance_resources(AppResources::from_world(ui_world))
                .with(game_screen_render_target(), render_target.0.clone());

            let task = async move {
                // NOTE: this is here to simulate a little connection delay to test the
                // robustness for execution before connect etc
                //
                // TODO: remove once the web is fully working
                tracing::info!("Sleeping for 2 seconds to simulate connection delay");
                sleep(Duration::from_millis(2000)).await;

                let conn = Connection::connect(url.clone()).await.with_context(|| {
                    format!("Failed to establish a WebTransport  session to {url}")
                })?;

                tracing::info!("Established WebTransport session");

                let (proxy_tx, proxy_rx) = flume::bounded(32);

                let mut proxy_tx = Some(WebTransportProxy::new(proxy_tx));
                // Create a handle for the game client

                handle_connection(
                    conn,
                    &assets,
                    user_id,
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
                        tracing::info!("Setting game state");
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

                tracing::info!("Finished handling connection");

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
            hooks.provide_context(|| client_state.clone());
            hooks
                .world
                .add_resource(crate::client::client_state(), Some(client_state.clone()));

            // FlowRow(vec![Text::el(format!("Connected")), inner]).el()
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

fn run_game_logic(
    hooks: &mut Hooks,
    game_state: SharedClientGameState,
    render_target: GameClientRenderTarget,
) {
    let world_event_reader = Mutex::new(hooks.world.resource(world_events()).reader());

    let gpu = hooks.world.resource(gpu()).clone();

    hooks.use_frame(move |app_world| {
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
    mut on_loaded: impl FnMut(&AssetCache, &str) -> anyhow::Result<(SharedClientGameState, CleanupFunc)>,
    control_rx: flume::Receiver<Control>,
    proxy_rx: flume::Receiver<ProxyMessage>,
) -> anyhow::Result<()> {
    tracing::info!("Handling client connection");
    tracing::info!("Opening control stream");

    let runtime = RuntimeKey.get(&assets);

    let mut request_send = FramedSendStream::new(conn.open_uni().await?);

    tracing::info!("Opened control stream");

    // Accept the diff and stat stream
    // Nothing is read from them until the connection has been accepted

    // Send a connection request
    tracing::info!("Attempting to connect using {user_id:?}");

    request_send
        .send(ClientRequest::Connect(user_id.clone()))
        .await?;

    let mut client = ClientProtoState::Pending(user_id.clone());

    tracing::info!("Accepting control stream from server");
    let mut push_recv = FramedRecvStream::new(
        conn.accept_uni()
            .await
            .ok_or(NetworkError::ConnectionClosed)??,
    );

    tracing::info!("Entering client loop");
    while client.is_pending() {
        tracing::info!("Waiting for server to accept connection and send server info");
        if let Some(frame) = push_recv.next().await {
            client.process_push(&assets, frame?)?;
        }
    }

    if !client.is_connected() {
        tracing::warn!("Connection failed or was denied");
        return Ok(());
    }

    tracing::info!("Accepting diff stream");
    let mut diff_stream = FramedRecvStream::new(
        conn.accept_uni()
            .await
            .ok_or(NetworkError::ConnectionClosed)??,
    );

    let (shared_client_state, cleanup) = on_loaded(&assets, &user_id)?;
    let on_disconnect = move || {
        tracing::info!("Running connection cleanup");
        cleanup()
    };

    scopeguard::defer!(on_disconnect());

    let mut control_rx = control_rx.into_stream();
    let mut proxy_rx = proxy_rx.into_stream();

    tracing::info!("Client connected");

    while let ClientProtoState::Connected(connected) = &mut client {
        tokio::select! {
            Some(frame) = push_recv.next() => {
                client.process_push(&assets, frame?)?;
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

    tracing::info!("Client entered disconnected state");
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
            tracing::info!("Sending bi request");
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
            tracing::info!("Sending uni request");
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
            tracing::info!("Sending datagram request");
            let mut bytes = BytesMut::with_capacity(4 + data.len());

            bytes.put_u32(id);
            bytes.put_slice(&data);

            let fut = conn.send_datagram(&bytes[..]);
            runtime.spawn_local(async move { log_network_result!(fut.await) });

            Ok(())
        }
    }
}
