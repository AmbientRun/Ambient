use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    future::Future,
    net::SocketAddr,
    ops::Deref,
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use ambient_app::window_title;
use ambient_core::{asset_cache, gpu, runtime, window::window_scale_factor};
use ambient_ecs::{
    components, generated::messages, world_events, ComponentValueBase, Entity, Resource,
    SystemGroup, World, WorldDiff,
};
use ambient_element::{element_component, Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_renderer::RenderTarget;
use ambient_rpc::RpcRegistry;
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::ContentBaseUrlKey,
    cb,
    fps_counter::FpsSample,
    friendly_id, to_byte_unit, CallbackFn, Cb,
};
use ambient_ui_native::{
    Button, Centered, FlowColumn, FlowRow, Image, MeasureSize, Text, Throbber,
};
use anyhow::Context;
use bytes::{BufMut, Bytes, BytesMut};
use futures::{future::BoxFuture, FutureExt, SinkExt, StreamExt, TryFutureExt};
use glam::{uvec2, UVec2};
use parking_lot::{const_rwlock, Mutex};
use quinn::Connection;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    time::sleep,
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug_span, Instrument};

use crate::{
    client_connection::ConnectionInner,
    client_game_state::ClientGameState,
    codec::FramedCodec,
    create_client_endpoint_random_port, log_network_result,
    proto::{
        client::{ClientState, SharedClientState},
        ClientControl, ServerControl,
    },
    protocol::{ClientInfo, ClientProtocol, ServerInfo},
    server::{self, FramedRecvStream, FramedSendStream, SharedServerState},
    stream::{self, RecvStream},
    NetworkError, MAX_FRAME_SIZE, RPC_BISTREAM_ID,
};

components!("network::client", {
    @[Resource]
    game_client: Option<GameClient>,
    @[Resource]
    bi_stream_handlers: BiStreamHandlers,
    @[Resource]
    uni_stream_handlers: UniStreamHandlers,
    @[Resource]
    datagram_handlers: DatagramHandlers,
});

pub type DynSend = Pin<Box<dyn AsyncWrite + Send + Sync>>;
pub type DynRecv = Pin<Box<dyn AsyncRead + Send + Sync>>;

type BiStreamHandler = Arc<dyn Fn(&mut World, AssetCache, DynSend, DynRecv) + Sync + Send>;
type UniStreamHandler = Arc<dyn Fn(&mut World, AssetCache, DynRecv) + Sync + Send>;
type DatagramHandler = Arc<dyn Fn(&mut World, AssetCache, Bytes) + Sync + Send>;

pub type BiStreamHandlers = HashMap<u32, (&'static str, BiStreamHandler)>;
pub type UniStreamHandlers = HashMap<u32, (&'static str, UniStreamHandler)>;
pub type DatagramHandlers = HashMap<u32, (&'static str, DatagramHandler)>;

/// A subset of the client state which allows for making transport agnostic RPCs and messages
/// without the hassles of associated types and non-object safety.
pub trait ClientConnection: 'static + Send + Sync {
    /// Performs a bidirectional request and waits for a response.
    fn request_bi(&self, id: u32, data: Bytes) -> BoxFuture<Result<Bytes, NetworkError>>;
    /// Performs a unidirectional request without waiting for a response.
    fn request_uni(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>>;
    fn send_datagram(&self, id: u32, data: Bytes) -> Result<(), NetworkError>;
}

impl ClientConnection for quinn::Connection {
    fn request_bi(&self, id: u32, data: Bytes) -> BoxFuture<Result<Bytes, NetworkError>> {
        Box::pin(async move {
            let (mut send, recv) = self.open_bi().await?;

            send.write_u32(id).await?;
            send.write_all(&data).await?;

            drop(send);

            let buf = recv.read_to_end(MAX_FRAME_SIZE).await?.into();

            Ok(buf)
        })
    }

    fn request_uni(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        Box::pin(async move {
            let mut send = self.open_uni().await?;

            send.write_u32(id).await?;
            send.write_all(&data).await?;

            Ok(())
        })
    }

    fn send_datagram(&self, id: u32, data: Bytes) -> Result<(), NetworkError> {
        let mut bytes = BytesMut::with_capacity(4 + data.len());
        bytes.put_u32(id);
        bytes.put(data);

        self.send_datagram(bytes.freeze())?;

        Ok(())
    }
}

#[derive(Clone)]
/// Manages the client side connection to the server.
pub struct GameClient {
    pub connection: Arc<dyn ClientConnection>,
    pub rpc_registry: Arc<RpcRegistry<server::RpcArgs>>,
    pub user_id: String,
    pub game_state: SharedClientState,
    pub uid: String,
}

impl Debug for GameClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameClient")
            .field("connection", &self.connection.type_name())
            .field("rpc_registry", &self.rpc_registry)
            .field("user_id", &self.user_id)
            .field("game_state", &self.game_state)
            .field("uid", &self.uid)
            .finish()
    }
}

impl GameClient {
    pub fn new(
        connection: Arc<dyn ClientConnection>,
        rpc_registry: Arc<RpcRegistry<server::RpcArgs>>,
        game_state: Arc<Mutex<ClientGameState>>,
        user_id: String,
    ) -> Self {
        Self {
            connection,
            rpc_registry,
            user_id,
            game_state,
            uid: friendly_id(),
        }
    }

    pub async fn rpc<
        Req: Serialize + DeserializeOwned + Send + 'static,
        Resp: Serialize + DeserializeOwned + Send,
        F: Fn(server::RpcArgs, Req) -> L + Send + Sync + Copy + 'static,
        L: Future<Output = Resp> + Send,
    >(
        &self,
        func: F,
        req: Req,
    ) -> Result<Resp, NetworkError> {
        rpc_request(&*self.connection, self.rpc_registry.clone(), func, req).await
    }

    pub fn make_standalone_rpc_wrapper<
        Req: Serialize + DeserializeOwned + Send + 'static,
        Resp: Serialize + DeserializeOwned + Send,
        F: Fn(server::RpcArgs, Req) -> L + Send + Sync + Copy + 'static,
        L: Future<Output = Resp> + Send,
    >(
        &self,
        runtime: &tokio::runtime::Handle,
        func: F,
    ) -> Cb<impl Fn(Req)> {
        let runtime = runtime.clone();
        let (connection, rpc_registry) = (self.connection.clone(), self.rpc_registry.clone());
        cb(move |req| {
            let (connection, rpc_registry) = (connection.clone(), rpc_registry.clone());
            runtime.spawn(async move {
                log_network_result!(rpc_request(&*connection, rpc_registry, func, req).await);
            });
        })
    }

    pub fn with_physics_world<R>(&self, f: impl Fn(&mut World) -> R) -> R {
        f(&mut self.game_state.lock().world)
    }
}

async fn rpc_request<
    Args: Send + 'static,
    Req: Serialize + DeserializeOwned + Send + 'static,
    Resp: Serialize + DeserializeOwned + Send,
    F: Fn(Args, Req) -> L + Send + Sync + Copy + 'static,
    L: Future<Output = Resp> + Send,
>(
    conn: &dyn ClientConnection,
    reg: Arc<RpcRegistry<Args>>,
    func: F,
    req: Req,
) -> Result<Resp, NetworkError> {
    let req = reg.serialize_req(func, req);

    let resp = conn.request_bi(RPC_BISTREAM_ID, req.into()).await?;

    let resp = reg.deserialize_resp(func, &resp)?;
    Ok(resp)
}

#[derive(Debug, Clone)]
pub struct GameClientRenderTarget(pub Arc<RenderTarget>);

#[derive(Debug)]
pub struct UseOnce<T> {
    val: Mutex<Option<T>>,
}

impl<T> UseOnce<T> {
    pub fn new(val: T) -> Self {
        Self {
            val: Mutex::new(Some(val)),
        }
    }

    pub fn take(&self) -> Option<T> {
        self.val.lock().take()
    }
}

pub type CleanupFunc = Box<dyn FnOnce() + Send + Sync>;
pub type LoadedFunc = Cb<dyn Fn(GameClient) -> anyhow::Result<CleanupFunc> + Send + Sync>;

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
pub struct GameClientView {
    pub server_addr: SocketAddr,
    pub user_id: String,
    pub systems_and_resources: Cb<dyn Fn() -> (SystemGroup, Entity) + Sync + Send>,
    pub error_view: Cb<dyn Fn(String) -> Element + Sync + Send>,
    pub on_loaded: LoadedFunc,
    pub on_in_entities: Option<Cb<dyn Fn(&WorldDiff) + Sync + Send>>,
    pub create_rpc_registry: Cb<dyn Fn() -> RpcRegistry<server::RpcArgs> + Sync + Send>,
    pub on_network_stats: Cb<dyn Fn(GameClientNetworkStats) + Sync + Send>,
    pub on_server_stats: Cb<dyn Fn(GameClientServerStats) + Sync + Send>,
    pub inner: Element,
}

impl ElementComponent for GameClientView {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self {
            server_addr,
            user_id,
            error_view,
            systems_and_resources,
            create_rpc_registry,
            on_loaded,
            on_in_entities,
            inner,
            on_network_stats,
            on_server_stats,
        } = *self;

        let gpu = hooks.world.resource(gpu()).clone();

        hooks.provide_context(|| {
            GameClientRenderTarget(Arc::new(RenderTarget::new(gpu.clone(), uvec2(1, 1), None)))
        });
        let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();

        let (connection_status, set_connection_status) = hooks.use_state("Connecting".to_string());

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

        // Subscribe to window close events
        hooks.use_runtime_message::<messages::WindowClose>({
            let game_client = game_client.clone();
            move |_, _| {
                tracing::info!("User closed the window");
                if let Some(game_client) = game_client.as_ref() {
                    todo!()
                    // game_client.connection.close(0u32.into(), b"User window was closed");
                }
            }
        });

        // Run game logic
        {
            let game_state = game_state.clone();
            let render_target = render_target.clone();
            let world_event_reader = Mutex::new(hooks.world.resource(world_events()).reader());

            let game_client_exists = game_client.is_some();
            hooks.use_frame(move |app_world| {
                if !game_client_exists {
                    return;
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

                // tracing::info!("Drawing game state");
                game_state.on_frame(&render_target.0);
            });
        }

        // Set the window title to the project name
        let (window_title_state, _set_window_title) = hooks.use_state("Ambient".to_string());
        *hooks.world.resource_mut(window_title()) = window_title_state;

        let (error, set_error) = hooks.use_state(None);

        hooks.use_task(move |_| {
            let open_and_run = async move {
                let conn = open_connection(server_addr)
                    .await
                    .with_context(|| format!("Failed to connect to endpoint: {server_addr:?}"))?;

                tracing::info!("Connected to the server");

                // Create a handle for the game client
                let game_client = GameClient::new(
                    Arc::new(conn.clone()),
                    Arc::new(create_rpc_registry()),
                    game_state.clone(),
                    user_id.clone(),
                );

                handle_connection(
                    game_client,
                    conn,
                    user_id,
                    ClientCallbacks {
                        on_stats: on_server_stats,
                        on_loaded: cb(move |game_client| {
                            let game_state = &game_client.game_state;
                            {
                                // Updates the game client context in the Ui tree
                                set_game_client(Some(game_client.clone()));
                                // Update the resources on the client side world to reflect the new connection
                                // state
                                let world = &mut game_state.lock().world;
                                world.add_resource(self::game_client(), Some(game_client.clone()));
                            }
                            (on_loaded)(game_client)
                        }),
                    },
                    game_state,
                )
                .await?;

                tracing::info!("Finished handling connection");

                Ok(()) as anyhow::Result<()>
            };

            async move {
                match open_and_run.await {
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

        if let Some(err) = error {
            return error_view(err);
        }

        if let Some(game_client) = game_client {
            // Provide the context
            hooks.provide_context(|| game_client.clone());
            hooks
                .world
                .add_resource(self::game_client(), Some(game_client.clone()));

            tracing::info!("Drawing inner");
            inner
        } else {
            Centered(vec![FlowColumn::el([FlowRow::el([
                Text::el(connection_status),
                Throbber.el(),
            ])])])
            .el()
        }
    }
}

#[derive(Debug)]
struct ClientCallbacks {
    on_loaded: LoadedFunc,
    on_stats: Cb<dyn Fn(GameClientServerStats) + Send + Sync>,
}

#[tracing::instrument(name = "client", level = "info", skip(conn))]
async fn handle_connection(
    game_client: GameClient,
    conn: quinn::Connection,
    user_id: String,
    callbacks: ClientCallbacks,
    state: SharedClientState,
) -> anyhow::Result<()> {
    tracing::info!("Handling client connection");
    tracing::info!("Opening control stream");

    let mut control_send = stream::SendStream::new(conn.open_uni().await?);

    tracing::info!("Opened control stream");

    // Accept the diff and stat stream
    // Nothing is read from them until the connection has been accepted

    // Send a connection request
    tracing::info!("Attempting to connect using {user_id:?}");

    control_send
        .send(&ServerControl::Connect(user_id.clone()))
        .await?;
    let mut client = ClientState::Connecting(user_id);

    tracing::info!("Accepting control stream from server");
    let mut control_recv = stream::RecvStream::new(conn.accept_uni().await?);

    tracing::info!("Entering client loop");
    while client.is_connecting() {
        tracing::info!("Waiting for server to accept connection and send server info");
        if let Some(frame) = control_recv.next().await {
            client.process_control(&state, frame?)?;
        }
    }

    tracing::info!("Accepting diff stream");
    let mut diff_stream = RecvStream::new(conn.accept_uni().await?);
    let on_disconnect = (callbacks.on_loaded)(game_client)?;
    scopeguard::defer!(on_disconnect());

    while let ClientState::Connected(connected) = &mut client {
        tokio::select! {
            Some(frame) = control_recv.next() => {
                client.process_control(&state, frame?)?;
            }
            // _ = stats_timer.tick() => {
            //     let stats = protocol.connection().stats();

            //     (self.on_client_stats)(GameClientNetworkStats {
            //         latency_ms: protocol.connection().rtt().as_millis() as u64,
            //         bytes_sent: (stats.udp_tx.bytes - prev_stats.udp_tx.bytes) / stats_interval,
            //         bytes_received: (stats.udp_rx.bytes - prev_stats.udp_rx.bytes) / stats_interval,
            //     });

            //     prev_stats = stats;
            // }

            Ok(datagram) = conn.read_datagram() => {
                connected.process_datagram(&state, datagram)?;
            }
            Ok((send, recv)) = conn.accept_bi() => {
                connected.process_bi(&state, send, recv).await?;
            }
            Ok(recv) = conn.accept_uni() => {
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

#[element_component]
pub fn GameClientWorld(hooks: &mut Hooks) -> Element {
    let (render_target, set_render_target) =
        hooks.consume_context::<GameClientRenderTarget>().unwrap();
    let gpu = hooks.world.resource(gpu()).clone();
    let scale_factor = *hooks.world.resource(window_scale_factor());
    MeasureSize::el(
        Image {
            texture: Some(Arc::new(
                render_target
                    .0
                    .color_buffer
                    .create_view(&Default::default()),
            )),
        }
        .el(),
        cb(move |size| {
            set_render_target(GameClientRenderTarget(Arc::new(RenderTarget::new(
                gpu.clone(),
                (size * scale_factor as f32).as_uvec2().max(UVec2::ONE),
                None,
            ))))
        }),
    )
}

struct ClientInstance<'a> {
    set_connection_status: CallbackFn<String>,
    server_addr: SocketAddr,
    user_id: String,

    /// Called when the client connected and received the world.
    on_init: &'a mut (dyn FnMut(
        Connection,
        ClientInfo,
        ServerInfo,
    ) -> anyhow::Result<Box<dyn FnOnce() + Sync + Send>>
                 + Send
                 + Sync),
    on_diff: &'a mut (dyn FnMut(WorldDiff) + Send + Sync),
    on_datagram: &'a (dyn Fn(u32, Bytes) + Send + Sync),
    on_bi_stream: &'a (dyn Fn(u32, quinn::SendStream, quinn::RecvStream) + Send + Sync),
    on_uni_stream: &'a (dyn Fn(u32, quinn::RecvStream) + Send + Sync),

    on_server_stats: &'a mut (dyn FnMut(GameClientServerStats) + Send + Sync),
    on_client_stats: &'a mut (dyn FnMut(GameClientNetworkStats) + Send + Sync),
    on_disconnect: Cb<dyn Fn() + Sync + Send + 'static>,
    init_destructor: Option<Box<dyn FnOnce() + Sync + Send>>,
}

impl<'a> Drop for ClientInstance<'a> {
    fn drop(&mut self) {
        (self.on_disconnect)();
        if let Some(on_disconnect) = self.init_destructor.take() {
            (on_disconnect)();
        }
    }
}

impl<'a> ClientInstance<'a> {
    #[tracing::instrument(skip(self))]
    async fn run(mut self) -> anyhow::Result<()> {
        log::info!("Connecting to server at {}", self.server_addr);
        (self.set_connection_status)(format!("Connecting to {}", self.server_addr));
        let conn = open_connection(self.server_addr).await?;

        (self.set_connection_status)("Waiting for server to respond".to_string());

        // Set up the protocol.
        let mut protocol = ClientProtocol::new(conn, self.user_id.clone()).await?;

        let stats_interval = 5;
        let mut stats_timer = tokio::time::interval(Duration::from_secs_f32(stats_interval as f32));
        let mut prev_stats = protocol.connection().stats();

        // The first WorldDiff initializes the world, so wait for that until we say things are "ready"
        (self.set_connection_status)("Receiving world".to_string());

        let msg = protocol.diff_stream.next().await?;
        (self.on_diff)(msg);
        self.init_destructor = Some(
            (self.on_init)(
                protocol.connection(),
                protocol.client_info().clone(),
                protocol.server_info.clone(),
            )
            .context("Client initialization failed")?,
        );

        // The server
        loop {
            tokio::select! {
                msg = protocol.diff_stream.next() => {
                    ambient_profiling::scope!("game_in_entities");
                    let msg: WorldDiff  = msg?;
                    (self.on_diff)(msg);
                }
                _ = stats_timer.tick() => {
                    let stats = protocol.connection().stats();

                    (self.on_client_stats)(GameClientNetworkStats {
                        latency_ms: protocol.connection().rtt().as_millis() as u64,
                        bytes_sent: (stats.udp_tx.bytes - prev_stats.udp_tx.bytes) / stats_interval,
                        bytes_received: (stats.udp_rx.bytes - prev_stats.udp_rx.bytes) / stats_interval,
                    });

                    prev_stats = stats;
                }
                Ok(stats) = protocol.stat_stream.next() => {
                    (self.on_server_stats)(GameClientServerStats(stats));
                }

                Ok(mut datagram) = protocol.conn.read_datagram() => {
                    let _span = tracing::debug_span!("datagram").entered();
                    let data = datagram.split_off(4);
                    let handler_id = u32::from_be_bytes(datagram[0..4].try_into().unwrap());
                    tokio::task::block_in_place(|| (self.on_datagram)(handler_id, data))
                }
                Ok((tx, mut rx)) = protocol.conn.accept_bi() => {
                    let span = tracing::debug_span!("bistream");
                    let stream_id = rx.read_u32().instrument(span).await;
                    if let Ok(stream_id) = stream_id {
                        tokio::task::block_in_place(|| { (self.on_bi_stream)(stream_id, tx, rx); })
                    }
                }
                Ok(mut rx) = protocol.conn.accept_uni() => {
                    let span = tracing::debug_span!("unistream");
                    let stream_id = rx.read_u32().instrument(span).await;
                    if let Ok(stream_id) = stream_id {
                        tokio::task::block_in_place(|| { (self.on_uni_stream)(stream_id, rx); })
                    }
                }
            }
        }
    }
}

/// Set up and manage a connection to the server
#[derive(Debug, Clone, Default)]
pub struct GameClientNetworkStats {
    pub latency_ms: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl Display for GameClientNetworkStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} ms rtt, {}/s out, {}/s in",
            self.latency_ms,
            to_byte_unit(self.bytes_sent),
            to_byte_unit(self.bytes_received)
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct GameClientServerStats(pub FpsSample);

/// Connnect to the server endpoint.
/// Does not handle a protocol.
#[tracing::instrument(level = "debug")]
pub async fn open_connection(server_addr: SocketAddr) -> anyhow::Result<Connection> {
    log::debug!("Connecting to world instance: {server_addr:?}");

    let endpoint =
        create_client_endpoint_random_port().context("Failed to create client endpoint")?;

    log::debug!("Got endpoint");
    let conn = endpoint.connect(server_addr, "localhost")?.await?;

    log::debug!("Got connection");
    Ok(conn)
}
