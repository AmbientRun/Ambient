use std::{
    collections::HashMap,
    fmt::Debug,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Range,
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use crate::{
    client::{DynRecv, DynSend},
    client_connection::ConnectionInner,
    codec::FramedCodec,
    connection::Connection,
    create_server, log_network_error, log_network_result,
    proto::{
        self,
        server::{handle_diffs, handle_stats, ConnectionData, Player},
        ClientControl, ServerControl,
    },
    protocol::{ClientInfo, ServerInfo},
    stream, NetworkError, OutgoingStream, ServerWorldExt, RPC_BISTREAM_ID,
};
use ambient_core::{
    asset_cache, name, no_sync,
    player::{get_by_user_id, player},
    project_name,
};
use ambient_ecs::{
    components, dont_store, query, ArchetypeFilter, ComponentDesc, Entity, EntityId, FrameEvent,
    Networked, Resource, System, SystemGroup, World, WorldDiff, WorldStream, WorldStreamCompEvent,
    WorldStreamFilter,
};
use ambient_proxy::client::AllocatedEndpoint;
use ambient_rpc::RpcRegistry;
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ServerBaseUrlKey},
    fps_counter::{FpsCounter, FpsSample},
    friendly_id, log_result,
};
use ambient_sys::time::{Instant, SystemTime};
use anyhow::bail;
use bytes::Bytes;
use colored::Colorize;
use flume::Sender;
use futures::{Sink, SinkExt, Stream, StreamExt};
use parking_lot::{Mutex, RwLock};
use quinn::{Endpoint, RecvStream, SendStream};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    time::{interval, MissedTickBehavior},
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug_span, Instrument};
use uuid::Uuid;

components!("network::server", {
    @[Resource]
    bi_stream_handlers: BiStreamHandlers,
    @[Resource]
    uni_stream_handlers: UniStreamHandlers,
    @[Resource]
    datagram_handlers: DatagramHandlers,

    player_entity_stream: Sender<Bytes>,
    player_connection_id: Uuid,
    @[Resource, Networked]
    server_stats: FpsSample,
});

pub type BiStreamHandler =
    Arc<dyn Fn(SharedServerState, AssetCache, &str, DynSend, DynRecv) + Sync + Send>;
pub type UniStreamHandler = Arc<dyn Fn(SharedServerState, AssetCache, &str, DynRecv) + Sync + Send>;
pub type DatagramHandler = Arc<dyn Fn(SharedServerState, AssetCache, &str, Bytes) + Sync + Send>;

pub type BiStreamHandlers = HashMap<u32, (&'static str, BiStreamHandler)>;
pub type UniStreamHandlers = HashMap<u32, (&'static str, UniStreamHandler)>;
pub type DatagramHandlers = HashMap<u32, (&'static str, DatagramHandler)>;

#[derive(Debug, Clone, Copy)]
pub struct ForkingEvent;

#[derive(Debug, Clone, Copy)]
pub struct ForkedEvent;

#[derive(Debug, Clone, Copy)]
pub struct ShutdownEvent;

pub struct WorldInstance {
    pub world: World,
    pub world_stream: WorldStream,
    pub systems: SystemGroup,
}

#[derive(Clone)]
pub struct RpcArgs {
    pub state: SharedServerState,
    pub user_id: String,
}
impl RpcArgs {
    pub fn get_player(&self, world: &World) -> Option<EntityId> {
        get_by_user_id(world, &self.user_id)
    }
}

pub fn create_player_entity_data(
    user_id: String,
    entities_tx: Sender<Bytes>,
    connection_id: Uuid,
) -> Entity {
    Entity::new()
        .with(name(), format!("Player {}", user_id))
        .with(ambient_core::player::player(), ())
        .with(ambient_core::player::user_id(), user_id)
        .with(player_entity_stream(), entities_tx)
        .with(player_connection_id(), connection_id)
        .with_default(dont_store())
}

pub fn register_rpc_bi_stream_handler(
    handlers: &mut BiStreamHandlers,
    rpc_registry: RpcRegistry<RpcArgs>,
) {
    handlers.insert(
        RPC_BISTREAM_ID,
        (
            "player_rpc",
            Arc::new(move |state, _assets, user_id, mut send, recv| {
                let state = state;
                let user_id = user_id.to_string();
                let rpc_registry = rpc_registry.clone();
                tokio::spawn(async move {
                    let try_block = || async {
                        let mut buf = Vec::new();
                        recv.take(1024 * 1024 * 1024).read_to_end(&mut buf).await?;
                        let args = RpcArgs {
                            state,
                            user_id: user_id.to_string(),
                        };
                        let resp = rpc_registry.run_req(args, &buf).await?;
                        send.write_all(&resp).await?;
                        // send.finish().await?;
                        Ok(()) as Result<(), NetworkError>
                    };
                    log_result!(try_block().await);
                });
            }),
        ),
    );
}

impl WorldInstance {
    /// Create server side player entity
    pub fn spawn_player(&mut self, ed: Entity) -> EntityId {
        ed.spawn(&mut self.world)
    }
    pub fn despawn_player(&mut self, user_id: &str) -> Option<Entity> {
        self.world.despawn(get_by_user_id(&self.world, user_id)?)
    }
    pub fn broadcast_diffs(&mut self) {
        let diff = self.world_stream.next_diff(&self.world);
        if diff.is_empty() {
            return;
        }
        let msg: Bytes = bincode::serialize(&diff).unwrap().into();

        ambient_profiling::scope!("Send MsgEntities");
        for (_, (entity_stream,)) in query((player_entity_stream(),)).iter(&self.world, None) {
            if let Err(_err) = entity_stream.send(msg.clone()) {
                log::warn!("Failed to broadcast diff to player");
            }
        }
    }
    pub fn player_count(&self) -> usize {
        query((player(),)).iter(&self.world, None).count()
    }
    pub fn step(&mut self, time: Duration) {
        self.world
            .set(self.world.resource_entity(), ambient_core::time(), time)
            .unwrap();
        self.systems.run(&mut self.world, &FrameEvent);
        self.world.next_frame();
    }
}

pub const MAIN_INSTANCE_ID: &str = "main";

pub type SharedServerState = Arc<Mutex<ServerState>>;

pub struct ServerState {
    pub assets: AssetCache,
    pub instances: HashMap<String, WorldInstance>,
    pub players: HashMap<String, Player>,
    pub create_server_systems: Arc<dyn Fn(&mut World) -> SystemGroup + Sync + Send>,
    pub create_on_forking_systems: Arc<dyn Fn() -> SystemGroup<ForkingEvent> + Sync + Send>,
    pub create_shutdown_systems: Arc<dyn Fn() -> SystemGroup<ShutdownEvent> + Sync + Send>,
}

impl ServerState {
    pub fn new_local(assets: AssetCache) -> Self {
        let world_stream_filter =
            WorldStreamFilter::new(ArchetypeFilter::new(), Arc::new(|_, _| false));
        Self {
            assets,
            instances: [(
                MAIN_INSTANCE_ID.to_string(),
                WorldInstance {
                    world: World::new("main_server"),
                    world_stream: WorldStream::new(world_stream_filter),
                    systems: SystemGroup::new("", vec![]),
                },
            )]
            .into(),
            players: Default::default(),
            create_server_systems: Arc::new(|_| SystemGroup::new("", vec![])),
            create_on_forking_systems: Arc::new(|| SystemGroup::new("", vec![])),
            create_shutdown_systems: Arc::new(|| SystemGroup::new("", vec![])),
        }
    }
    pub fn new(
        assets: AssetCache,
        instances: HashMap<String, WorldInstance>,
        create_server_systems: Arc<dyn Fn(&mut World) -> SystemGroup + Sync + Send>,
        create_on_forking_systems: Arc<dyn Fn() -> SystemGroup<ForkingEvent> + Sync + Send>,
        create_shutdown_systems: Arc<dyn Fn() -> SystemGroup<ShutdownEvent> + Sync + Send>,
    ) -> Self {
        Self {
            assets,
            instances,
            players: Default::default(),
            create_server_systems,
            create_on_forking_systems,
            create_shutdown_systems,
        }
    }

    pub fn step(&mut self) {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        for instance in self.instances.values_mut() {
            instance.step(time);
        }
    }
    pub fn broadcast_diffs(&mut self) {
        for instance in self.instances.values_mut() {
            instance.broadcast_diffs();
        }
    }
    pub fn player_count(&self) -> usize {
        self.instances.values().map(|i| i.player_count()).sum()
    }
    pub fn get_player_world_instance_mut(&mut self, user_id: &str) -> Option<&mut WorldInstance> {
        self.players
            .get(user_id)
            .and_then(|player| self.instances.get_mut(&player.instance))
    }
    pub fn get_player_world_instance(&self, user_id: &str) -> Option<&WorldInstance> {
        self.players
            .get(user_id)
            .and_then(|player| self.instances.get(&player.instance))
    }
    pub fn get_player_world_mut(&mut self, user_id: &str) -> Option<&mut World> {
        self.get_player_world_instance_mut(user_id)
            .map(|i| &mut i.world)
    }
    pub fn get_player_world(&self, user_id: &str) -> Option<&World> {
        self.get_player_world_instance(user_id).map(|i| &i.world)
    }
    pub fn remove_instance(&mut self, instance_id: &str) {
        log::debug!("Removing server instance id={}", instance_id);
        let mut sys = (self.create_shutdown_systems)();
        let old_instance = self.instances.get_mut(instance_id).unwrap();
        sys.run(&mut old_instance.world, &ShutdownEvent);
        self.instances.remove(instance_id);
    }
}

#[derive(Debug, Clone)]
pub struct ProxySettings {
    pub endpoint: String,
    pub project_path: AbsAssetUrl,
    pub pre_cache_assets: bool,
    pub project_id: String,
}

pub struct GameServer {
    endpoint: Endpoint,
    pub port: u16,
    /// Shuts down the server if there are no players
    pub use_inactivity_shutdown: bool,
    proxy_settings: Option<ProxySettings>,
}
impl GameServer {
    pub async fn new_with_port(
        port: u16,
        use_inactivity_shutdown: bool,
        proxy_settings: Option<ProxySettings>,
    ) -> anyhow::Result<Self> {
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

        let endpoint = create_server(server_addr)?;

        log::debug!("GameServer listening on port {}", port);
        Ok(Self {
            endpoint,
            port,
            use_inactivity_shutdown,
            proxy_settings,
        })
    }
    pub async fn new_with_port_in_range(
        port_range: Range<u16>,
        use_inactivity_shutdown: bool,
        proxy_settings: Option<ProxySettings>,
    ) -> anyhow::Result<Self> {
        for port in port_range {
            match Self::new_with_port(port, use_inactivity_shutdown, proxy_settings.clone()).await {
                Ok(server) => {
                    return Ok(server);
                }
                Err(_err) => {
                    log::warn!("Failed to create server on port {}", port);
                }
            }
        }
        bail!("Failed to create server")
    }
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn run(
        self,
        mut world: World,
        create_server_systems: Arc<dyn Fn(&mut World) -> SystemGroup + Sync + Send>,
        create_on_forking_systems: Arc<dyn Fn() -> SystemGroup<ForkingEvent> + Sync + Send>,
        create_shutdown_systems: Arc<dyn Fn() -> SystemGroup<ShutdownEvent> + Sync + Send>,
        is_sync_component: Arc<dyn Fn(ComponentDesc, WorldStreamCompEvent) -> bool + Sync + Send>,
    ) -> SharedServerState {
        let Self {
            endpoint,
            proxy_settings,
            ..
        } = self;
        let assets = world.resource(asset_cache()).clone();
        let world_stream_filter =
            WorldStreamFilter::new(ArchetypeFilter::new().excl(no_sync()), is_sync_component);
        let state = Arc::new(Mutex::new(ServerState::new(
            assets.clone(),
            [(
                MAIN_INSTANCE_ID.to_string(),
                WorldInstance {
                    systems: create_server_systems(&mut world),
                    world,
                    world_stream: WorldStream::new(world_stream_filter.clone()),
                },
            )]
            .into_iter()
            .collect(),
            create_server_systems,
            create_on_forking_systems,
            create_shutdown_systems,
        )));

        let mut fps_counter = FpsCounter::new();
        let mut sim_interval = interval(Duration::from_secs_f32(1. / 60.));
        sim_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut inactivity_interval = interval(Duration::from_secs_f32(5.));
        let mut last_active = ambient_sys::time::Instant::now();

        if let Some(proxy_settings) = proxy_settings {
            let endpoint = endpoint.clone();
            let state = state.clone();
            let world_stream_filter = world_stream_filter.clone();
            let assets = assets.clone();
            tokio::spawn(async move {
                start_proxy_connection(
                    endpoint.clone(),
                    proxy_settings,
                    state.clone(),
                    world_stream_filter.clone(),
                    assets.clone(),
                )
                .await;
            });
        }

        loop {
            tracing::trace_span!("Listening for incoming connections");
            tokio::select! {
                Some(conn) = endpoint.accept() => {
                    log::debug!("Received connection");

                    let conn = match conn.await {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("Failed to accept incoming connection. {e}");
                            continue;
                        }
                    };


                    log::debug!("Accepted connection");
                    let fut = handle_connection(conn.into(), state.clone(), world_stream_filter.clone(), assets.clone(), ServerBaseUrlKey.get(&assets));
                    tokio::spawn(async move {  log_result!(fut.await) });
                }
                _ = sim_interval.tick() => {
                    fps_counter.frame_start();
                    let mut state = state.lock();
                    tokio::task::block_in_place(|| {
                        ambient_profiling::finish_frame!();
                        ambient_profiling::scope!("sim_tick");
                        state.step();
                        state.broadcast_diffs();
                        if let Some(sample) = fps_counter.frame_end() {
                            for instance in state.instances.values_mut() {
                                let id = instance.world.synced_resource_entity().unwrap();
                                instance.world.add_component(id,server_stats(), sample.clone());
                            }
                        }
                    });
                }
                _ = inactivity_interval.tick(), if self.use_inactivity_shutdown => {
                    if state.lock().player_count() == 0 {
                        if Instant::now().duration_since(last_active).as_secs_f32() > 2. * 60. {
                            log::info!("[{}] Shutting down due to inactivity", self.port);
                            break;
                        }
                    } else {
                        last_active = Instant::now();
                    }
                }
                else => {
                    log::info!("No more connections. Shutting down.");
                    break
                }
            }
        }
        log::debug!("[{}] GameServer shutting down", self.port);
        {
            let mut state = state.lock();
            let create_shutdown_systems = state.create_shutdown_systems.clone();
            for instance in state.instances.values_mut() {
                let mut sys = (create_shutdown_systems)();
                sys.run(&mut instance.world, &ShutdownEvent);
            }
        }
        log::debug!("[{}] GameServer finished shutting down", self.port);
        state
    }
}

async fn start_proxy_connection(
    endpoint: Endpoint,
    settings: ProxySettings,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    assets: AssetCache,
) {
    // start with content base url being the same as for direct connections
    let content_base_url = Arc::new(RwLock::new(ServerBaseUrlKey.get(&assets)));

    let on_endpoint_allocated = {
        let content_base_url = content_base_url.clone();
        Arc::new(
            move |AllocatedEndpoint {
                      id,
                      allocated_endpoint,
                      external_endpoint,
                      assets_root,
                      ..
                  }: AllocatedEndpoint| {
                log::debug!("Allocated proxy endpoint. Allocation id: {}", id);
                log::info!("Proxy sees this server as {}", external_endpoint);
                log::info!(
                    "Proxy allocated an endpoint, use `{}` to join",
                    format!("ambient join {}", allocated_endpoint).bright_green()
                );

                // set the content base url to point to proxy provided value
                match AbsAssetUrl::parse(&assets_root) {
                    Ok(url) => {
                        log::debug!("Got content base root from proxy: {}", url);
                        *content_base_url.write() = url;
                    }
                    Err(err) => {
                        log::warn!("Failed to parse assets root url ({}): {}", assets_root, err)
                    }
                }
            },
        )
    };

    let on_player_connected = {
        let assets = assets.clone();
        let content_base_url = content_base_url.clone();
        Arc::new(
            move |_player_id, conn: ambient_proxy::client::ProxiedConnection| {
                log::debug!("Accepted connection via proxy");
                let task = handle_connection(
                    conn.into(),
                    state.clone(),
                    world_stream_filter.clone(),
                    assets.clone(),
                    content_base_url.read().clone(),
                );

                tokio::spawn(async move { log_result!(task.await) });
            },
        )
    };

    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

    let builder = ambient_proxy::client::builder()
        .endpoint(endpoint.clone())
        .proxy_server(settings.endpoint.clone())
        .project_id(settings.project_id.clone())
        .user_agent(APP_USER_AGENT.to_string());

    let assets_path = settings
        .project_path
        .push("build")
        .expect("Pushing to path cannot fail");
    let builder = if let Ok(Some(assets_file_path)) = assets_path.to_file_path() {
        builder.assets_path(assets_file_path)
    } else {
        builder.assets_root_override(content_base_url.read().to_string())
    };

    log::info!("Connecting to proxy server");
    let proxy = match builder.build().await {
        Ok(proxy_client) => proxy_client,
        Err(err) => {
            log::warn!("Failed to connect to proxy: {}", err);
            return;
        }
    };

    // start and allocate endpoint
    let mut controller = proxy.start(on_endpoint_allocated, on_player_connected);
    log::info!("Allocating proxy endpoint");
    if let Err(err) = controller.allocate_endpoint().await {
        log::warn!("Failed to allocate proxy endpoint: {}", err);
    }

    // pre-cache "assets" subdirectory
    if settings.pre_cache_assets {
        for subdir in ["assets", "client"] {
            if let Err(err) = controller.pre_cache_assets(subdir) {
                log::warn!("Failed to pre-cache assets: {}", err);
            }
        }
    }
}

pub type FramedRecvStream<T> = FramedRead<RecvStream, FramedCodec<T>>;
pub type FramedSendStream<T> = FramedWrite<SendStream, FramedCodec<T>>;

/// Setup the protocol and enter the update loop for a new connected client
#[tracing::instrument(
    name = "server",
    level = "info",
    skip(conn, state, world_stream_filter, assets, content_base_url)
)]
async fn handle_connection(
    conn: ConnectionInner,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    assets: AssetCache,
    content_base_url: AbsAssetUrl,
) -> anyhow::Result<()> {
    tracing::info!("Handling server connection");
    // let handle = Arc::new(OnceCell::new());
    // handle
    //     .set({
    //         let handle = handle.clone();
    //         tokio::spawn(async move {
    let (diffs_tx, diffs_rx) = flume::unbounded();

    let server_info = {
        let state = state.lock();
        let instance = state.instances.get(MAIN_INSTANCE_ID).unwrap();
        let world = &instance.world;
        ServerInfo {
            project_name: world.resource(project_name()).clone(),
            content_base_url,
            ..Default::default()
        }
    };

    let mut server = proto::server::ServerState::default();

    tracing::info!("Accepting control stream from client");
    let mut control_recv = stream::RecvStream::new(conn.accept_uni().await?);
    tracing::info!("Opening control stream");
    let mut control_send = stream::SendStream::new(conn.open_uni().await?);

    let diffs_rx = diffs_rx.into_stream();

    use futures::SinkExt;

    // Send who we are
    control_send
        .send(ClientControl::ServerInfo(server_info))
        .await?;

    // Feed the channel senders to the connection data
    //
    // Once connected they will be added to the player entity
    let data = ConnectionData {
        state,
        diff_tx: diffs_tx,
        connection_id: Uuid::new_v4(),
        world_stream_filter,
    };

    while server.is_pending_connection() {
        tracing::info!("Waiting for connect request");
        if let Some(frame) = control_recv.next().await {
            server.process_control(&data, frame?)?;
        }
    }

    tracing::debug!("Performing additional on connect logic after the fact");

    tokio::spawn(handle_diffs(
        stream::SendStream::new(conn.open_uni().await?),
        diffs_rx,
    ));

    // Before a connection has been established, only process the control stream
    while let proto::server::ServerState::Connected(connected) = &mut server {
        tokio::select! {
            Some(frame) = control_recv.next() => {
                server.process_control(&data, frame?)?;
            }
            stream = conn.accept_uni() => {
                connected.process_uni(&data, stream?).await?;
            }
            stream = conn.accept_bi() => {
                let (send, recv) = stream?;
                connected.process_bi(&data, send, recv).await?;
            }
            datagram = conn.read_datagram() => {
                connected.process_datagram(&data, datagram?).await?;
            }
            Some(msg) = connected.control_rx.next() => {
                let span = tracing::debug_span!("internal control");
                control_send.send(&msg).instrument(span).await?;
            }
        }
    }

    tracing::info!("Client disconnected");

    Ok(())
}
