use ambient_core::{asset_cache, gpu, RuntimeKey};
use ambient_ecs::{world_events, Entity, SystemGroup};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_renderer::RenderTarget;
use ambient_rpc::RpcRegistry;
use ambient_std::{asset_cache::SyncAssetKeyExt, cb, Cb};
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
    client::{Control, GameClient, GameClientRenderTarget, LoadedFunc},
    client_game_state::ClientGameState,
    log_network_result,
    proto::{
        client::{ClientState, SharedClientState},
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
            GameClientRenderTarget(Arc::new(RenderTarget::new(gpu.clone(), uvec2(1, 1), None)))
        });
        let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();

        let assets = hooks.world.resource(asset_cache()).clone();
        let game_state = hooks.use_ref_with(|world| {
            let (systems, resources) = systems_and_resources();

            ClientGameState::new(
                world,
                assets.clone(),
                user_id.clone(),
                render_target.0.clone(),
                systems,
                resources,
            )
        });

        // The game client will be set once a connection establishes
        let (game_client, set_game_client) = hooks.use_state(None as Option<GameClient>);

        // When the client is connected, run the update logic each frame
        if game_client.is_some() {
            run_game_logic(hooks, game_state.clone(), render_target);
        }

        // TODO: allow remote shutdown
        let ((_control_tx, control_rx), _) = hooks.use_state_with(|_| flume::unbounded());

        let (err, set_error) = hooks.use_state(None);

        hooks.use_local_task(|_| {
            let task = async move {
                sleep(Duration::from_millis(2000)).await;
                let conn = Connection::connect(url.clone()).await.with_context(|| {
                    format!("Failed to establish a WebTransport  session to {url:?}")
                })?;

                tracing::info!("Established WebTransport session");

                let (tx, proxy_rx) = flume::bounded(16);
                let proxy = WebTransportProxy::new(tx);

                // Create a handle for the game client
                let game_client = GameClient::new(
                    Arc::new(proxy),
                    Arc::new(create_rpc_registry()),
                    game_state.clone(),
                    user_id.clone(),
                );

                handle_connection(
                    game_client,
                    conn,
                    user_id,
                    cb(move |game_client| {
                        let game_state = &game_client.game_state;
                        {
                            // Updates the game client context in the Ui tree
                            set_game_client(Some(game_client.clone()));
                            // Update the resources on the client side world to reflect the new connection
                            // state
                            let world = &mut game_state.lock().world;
                            world.add_resource(
                                crate::client::game_client(),
                                Some(game_client.clone()),
                            );
                        }
                        (on_loaded)(game_client)
                    }),
                    game_state,
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

        if let Some(game_client) = game_client {
            // Provide the context
            hooks.provide_context(|| game_client.clone());
            hooks
                .world
                .add_resource(crate::client::game_client(), Some(game_client.clone()));

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
    game_state: SharedClientState,
    render_target: GameClientRenderTarget,
) {
    let world_event_reader = Mutex::new(hooks.world.resource(world_events()).reader());

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

        // tracing::info!("Drawing game state");
        game_state.on_frame(&render_target.0);
    });
}

async fn handle_connection(
    game_client: GameClient,
    mut conn: Connection,
    user_id: String,
    on_loaded: LoadedFunc,
    state: SharedClientState,
    control_rx: flume::Receiver<Control>,
    proxy_rx: flume::Receiver<ProxyMessage>,
) -> anyhow::Result<()> {
    tracing::info!("Handling client connection");
    tracing::info!("Opening control stream");
    let assets = state.lock().assets.clone();
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

    let mut client = ClientState::Connecting(user_id);

    tracing::info!("Accepting control stream from server");
    let mut push_recv = FramedRecvStream::new(
        conn.accept_uni()
            .await
            .ok_or(NetworkError::ConnectionClosed)??,
    );

    tracing::info!("Entering client loop");
    while client.is_connecting() {
        tracing::info!("Waiting for server to accept connection and send server info");
        if let Some(frame) = push_recv.next().await {
            client.process_push(&state, frame?)?;
        }
    }

    tracing::info!("Accepting diff stream");
    let mut diff_stream = FramedRecvStream::new(
        conn.accept_uni()
            .await
            .ok_or(NetworkError::ConnectionClosed)??,
    );

    let cleanup = on_loaded(game_client)?;
    let on_disconnect = move || {
        tracing::info!("Running connection cleanup");
        cleanup()
    };

    scopeguard::defer!(on_disconnect());

    let mut control_rx = control_rx.into_stream();
    let mut proxy_rx = proxy_rx.into_stream();

    tracing::info!("Client connected");

    while let ClientState::Connected(connected) = &mut client {
        tokio::select! {
            Some(frame) = push_recv.next() => {
                client.process_push(&state, frame?)?;
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
                connected.process_datagram(&state, datagram)?;
            }

            Some(Ok((send, recv))) = conn.accept_bi() => {
                connected.process_bi(&state, send, recv).await?;
            }

            Some(Ok(recv)) = conn.accept_uni() => {
                connected.process_uni(&state, recv).await?;
            }

            Some(diff) = diff_stream.next() => {
                connected.process_diff(&state, diff?)?;
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
