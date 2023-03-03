use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Range,
    sync::Arc,
    time::Duration,
};

use ambient_core::{asset_cache, no_sync, project_name};
use ambient_ecs::{
    components, dont_store, query, ArchetypeFilter, ComponentDesc, Entity, EntityId, FrameEvent, System, SystemGroup, World, WorldStream,
    WorldStreamCompEvent, WorldStreamFilter,
};
use ambient_std::{
    asset_cache::AssetCache,
    fps_counter::{FpsCounter, FpsSample},
    friendly_id, log_result,
};
use ambient_sys::time::{Instant, SystemTime};
use anyhow::bail;
use bytes::Bytes;
use flume::Sender;
use futures::StreamExt;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use quinn::{Endpoint, Incoming, NewConnection, RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncReadExt,
    time::{interval, MissedTickBehavior},
};
use tracing::{debug_span, Instrument};

use crate::{
    bi_stream_handlers, create_server, datagram_handlers, get_player_by_user_id, player,
    protocol::{ClientInfo, ServerProtocol},
    NetworkError,
};

components!("network", {
    player_entity_stream: Sender<Vec<u8>>,
    player_event_stream: Sender<Vec<u8>>,
    player_stats_stream: Sender<FpsSample>,
});

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

pub fn create_player_entity_data(
    user_id: &str,
    entities_tx: Sender<Vec<u8>>,
    events_tx: Sender<Vec<u8>>,
    stats_tx: Sender<FpsSample>,
) -> Entity {
    Entity::new()
        .with(crate::player::player(), ())
        .with(crate::player::user_id(), user_id.to_string())
        .with(player_entity_stream(), entities_tx)
        .with(player_stats_stream(), stats_tx)
        .with(player_event_stream(), events_tx)
        .with_default(dont_store())
}

impl WorldInstance {
    /// Create server side player entity
    pub fn spawn_player(&mut self, ed: Entity) -> EntityId {
        ed.spawn(&mut self.world)
    }
    pub fn despawn_player(&mut self, user_id: &str) -> Option<Entity> {
        self.world.despawn(get_player_by_user_id(&self.world, user_id)?)
    }
    pub fn broadcast_diffs(&mut self) {
        let diff = self.world_stream.next_diff(&self.world);
        if diff.is_empty() {
            return;
        }
        let msg = bincode::serialize(&diff).unwrap();

        profiling::scope!("Send MsgEntities");
        for (_, (entity_stream,)) in query((player_entity_stream(),)).iter(&self.world, None) {
            let msg = msg.clone();
            if let Err(_err) = entity_stream.send(msg) {
                log::warn!("Failed to broadcast diff to player");
            }
        }
    }
    pub fn player_count(&self) -> usize {
        query((player(),)).iter(&self.world, None).count()
    }
    pub fn step(&mut self, time: Duration) {
        self.world.set(self.world.resource_entity(), ambient_core::time(), time).unwrap();
        self.systems.run(&mut self.world, &FrameEvent);
        self.world.next_frame();
    }
}

pub const MAIN_INSTANCE_ID: &str = "main";

pub struct Player {
    pub instance: String,
    pub abort_handle: Arc<OnceCell<tokio::task::JoinHandle<()>>>,
    pub connection_id: String,
}

impl Player {
    pub fn new(instance: String, abort_handle: Arc<OnceCell<tokio::task::JoinHandle<()>>>, connection_id: String) -> Self {
        Self { instance, abort_handle, connection_id }
    }

    pub fn new_local(instance: String) -> Self {
        Self { instance, abort_handle: Arc::new(OnceCell::new()), connection_id: friendly_id() }
    }
}

pub type SharedServerState = Arc<Mutex<ServerState>>;
pub struct ServerState {
    pub instances: HashMap<String, WorldInstance>,
    pub players: HashMap<String, Player>,
    pub create_server_systems: Arc<dyn Fn(&mut World) -> SystemGroup + Sync + Send>,
    pub create_on_forking_systems: Arc<dyn Fn() -> SystemGroup<ForkingEvent> + Sync + Send>,
    pub create_shutdown_systems: Arc<dyn Fn() -> SystemGroup<ShutdownEvent> + Sync + Send>,
}
impl ServerState {
    pub fn new_local() -> Self {
        let world_stream_filter = WorldStreamFilter::new(ArchetypeFilter::new(), Arc::new(|_, _| false));
        Self {
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
        instances: HashMap<String, WorldInstance>,
        create_server_systems: Arc<dyn Fn(&mut World) -> SystemGroup + Sync + Send>,
        create_on_forking_systems: Arc<dyn Fn() -> SystemGroup<ForkingEvent> + Sync + Send>,
        create_shutdown_systems: Arc<dyn Fn() -> SystemGroup<ShutdownEvent> + Sync + Send>,
    ) -> Self {
        Self { instances, players: Default::default(), create_server_systems, create_on_forking_systems, create_shutdown_systems }
    }

    pub fn step(&mut self) {
        let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
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
        self.players.get(user_id).and_then(|player| self.instances.get_mut(&player.instance))
    }
    pub fn get_player_world_instance(&self, user_id: &str) -> Option<&WorldInstance> {
        self.players.get(user_id).and_then(|player| self.instances.get(&player.instance))
    }
    pub fn get_player_world_mut(&mut self, user_id: &str) -> Option<&mut World> {
        self.get_player_world_instance_mut(user_id).map(|i| &mut i.world)
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

pub struct GameServer {
    _endpoint: Endpoint,
    incoming: Incoming,
    pub port: u16,
    /// Shuts down the server if there are no players
    pub use_inactivity_shutdown: bool,
}
impl GameServer {
    pub async fn new_with_port(port: u16, use_inactivity_shutdown: bool) -> anyhow::Result<Self> {
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

        let (endpoint, incoming) = create_server(server_addr)?;

        log::debug!("GameServer listening on port {}", port);
        Ok(Self { _endpoint: endpoint, incoming, port, use_inactivity_shutdown })
    }
    pub async fn new_with_port_in_range(port_range: Range<u16>, use_inactivity_shutdown: bool) -> anyhow::Result<Self> {
        for port in port_range {
            match Self::new_with_port(port, use_inactivity_shutdown).await {
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
    #[tracing::instrument(skip_all)]
    pub async fn run(
        self,
        mut world: World,
        create_server_systems: Arc<dyn Fn(&mut World) -> SystemGroup + Sync + Send>,
        create_on_forking_systems: Arc<dyn Fn() -> SystemGroup<ForkingEvent> + Sync + Send>,
        create_shutdown_systems: Arc<dyn Fn() -> SystemGroup<ShutdownEvent> + Sync + Send>,
        is_sync_component: Arc<dyn Fn(ComponentDesc, WorldStreamCompEvent) -> bool + Sync + Send>,
    ) -> SharedServerState {
        let Self { mut incoming, .. } = self;
        let assets = world.resource(asset_cache()).clone();
        let world_stream_filter = WorldStreamFilter::new(ArchetypeFilter::new().excl(no_sync()), is_sync_component);
        let state = Arc::new(Mutex::new(ServerState::new(
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

        loop {
            tracing::debug_span!("Listening for incoming connections");
            tokio::select! {
                Some(conn) = incoming.next() => {
                    log::debug!("Received connection");

                    let conn = match conn.await {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("Failed to accept incoming connection. {e}");
                            continue;
                        }
                    };


                    log::debug!("Accepted connection");
                    run_connection(conn, state.clone(), world_stream_filter.clone(), assets.clone());
                }
                _ = sim_interval.tick() => {
                    fps_counter.frame_start();
                    let mut state = state.lock();
                    tokio::task::block_in_place(|| {
                        profiling::finish_frame!();
                        profiling::scope!("sim_tick");
                        state.step();
                        state.broadcast_diffs();
                        if let Some(sample) = fps_counter.frame_end() {
                            for instance in state.instances.values() {
                                for (_, (stream,)) in query((player_stats_stream(),)).iter(&instance.world, None) {
                                    stream.send(sample.clone()).ok();
                                }
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

/// Setup the protocol and enter the update loop for a new connected client
#[tracing::instrument(skip_all)]
fn run_connection(connection: NewConnection, state: SharedServerState, world_stream_filter: WorldStreamFilter, assets: AssetCache) {
    let connection_id = friendly_id();
    let handle = Arc::new(OnceCell::new());
    handle
        .set({
            let handle = handle.clone();
            tokio::spawn(async move {
                let (diffs_tx, diffs_rx) = flume::unbounded();
                let (stats_tx, stats_rx) = flume::unbounded();
                let (events_tx, events_rx) = flume::unbounded();

                let on_init = |client: ClientInfo| {
                    let user_id = &client.user_id;
                    log::debug!("[{}] Locking world", user_id);
                    let mut state = state.lock();
                    // If there's an old player
                    let reconnecting = if let Some(player) = state.players.get_mut(user_id) {
                        if let Some(handle) = player.abort_handle.get() {
                            handle.abort();
                        }
                        player.abort_handle = handle.clone();
                        player.connection_id = connection_id.clone();
                        log::debug!("[{}] Player reconnecting", user_id);
                        true
                    } else {
                        state.players.insert(
                            user_id.clone(),
                            Player {
                                instance: MAIN_INSTANCE_ID.to_string(),
                                abort_handle: handle.clone(),
                                connection_id: connection_id.clone(),
                            },
                        );
                        false
                    };

                    let instance = state.instances.get_mut(MAIN_INSTANCE_ID).unwrap();

                    // Bring world stream up to the current time
                    log::debug!("[{}] Broadcasting diffs", user_id);
                    instance.broadcast_diffs();
                    log::debug!("[{}] Creating init diff", user_id);

                    let diff = world_stream_filter.initial_diff(&instance.world);
                    let diff = bincode::serialize(&diff).unwrap();

                    log_result!(diffs_tx.send(diff));
                    log::debug!("[{}] Init diff sent", user_id);

                    if !reconnecting {
                        instance.spawn_player(create_player_entity_data(user_id, diffs_tx.clone(), events_tx.clone(), stats_tx.clone()));
                        log::info!("[{}] Player spawned", user_id);
                    } else {
                        let entity = get_player_by_user_id(&instance.world, user_id).unwrap();
                        instance.world.set(entity, player_entity_stream(), diffs_tx.clone()).unwrap();
                        instance.world.set(entity, player_stats_stream(), stats_tx.clone()).unwrap();
                        instance.world.set(entity, player_event_stream(), events_tx.clone()).unwrap();
                        log::info!("[{}] Player reconnected", user_id);
                    }
                };

                let on_disconnect = |user_id: &Option<String>| {
                    if let Some(user_id) = user_id {
                        log::debug!("[{}] Disconnecting", user_id);
                        let mut state = state.lock();
                        if state.players.get(user_id).map(|p| p.connection_id != connection_id).unwrap_or(false) {
                            log::info!("[{}] Disconnected (reconnection)", user_id);
                            return;
                        }
                        if let Some(player) = state.players.remove(user_id) {
                            state.instances.get_mut(&player.instance).unwrap().despawn_player(user_id);
                        }

                        log::info!("[{}] Disconnected", user_id);
                    }
                };

                let on_rpc = |user_id: &String, stream_id, tx, rx| {
                    let _span = debug_span!("on_rpc").entered();
                    let handler = {
                        let state = state.lock();
                        let world = match state.get_player_world(user_id) {
                            Some(world) => world,
                            None => {
                                log::error!("Player missing for rpc."); // Probably disconnected
                                return;
                            }
                        };

                        world.resource(bi_stream_handlers()).get(&stream_id).cloned()
                    };
                    if let Some(handler) = handler {
                        handler(state.clone(), assets.clone(), user_id, tx, rx);
                    } else {
                        log::error!("Unrecognized stream type: {}", stream_id);
                    }
                };

                let on_datagram = |user_id: &String, mut bytes: Bytes| {
                    let data = bytes.split_off(4);
                    let handler_id = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
                    let state = state.clone();
                    let handler = {
                        let state = state.lock();
                        let world = match state.get_player_world(user_id) {
                            Some(world) => world,
                            None => {
                                log::warn!("Player missing for datagram."); // Probably disconnected
                                return;
                            }
                        };
                        world.resource(datagram_handlers()).get(&handler_id).cloned()
                    };
                    match handler {
                        Some(handler) => {
                            handler(state, assets.clone(), user_id, data);
                        }
                        None => {
                            log::error!("No such datagram handler: {:?}", handler_id);
                        }
                    }
                };

                let client = ClientInstance {
                    diffs_rx,
                    stats_rx,
                    events_rx,
                    on_init: &on_init,
                    on_rpc: &on_rpc,
                    on_datagram: &on_datagram,
                    on_disconnect: &on_disconnect,
                    user_id: None,
                };

                let server_info = {
                    let state = state.lock();
                    let instance = state.instances.get(MAIN_INSTANCE_ID).unwrap();
                    let world = &instance.world;
                    ServerInfo { project_name: world.resource(project_name()).clone() }
                };

                match client.run(connection, server_info).await {
                    Ok(()) => {}
                    Err(err) if err.is_closed() => {
                        log::info!("Connection closed by client");
                    }
                    Err(err) if err.is_end_of_stream() => {
                        log::warn!("Stream was closed prematurely");
                    }
                    Err(NetworkError::IOError(err)) if err.kind() == std::io::ErrorKind::NotConnected => {
                        log::warn!("Not connected: {err:?}");
                    }
                    Err(err) => {
                        log::error!("Server error: {err:?}");
                    }
                };
            })
        })
        .expect("Player handle set twice");
}

/// Manages the server side client communication
struct ClientInstance<'a> {
    diffs_rx: flume::Receiver<Vec<u8>>,
    stats_rx: flume::Receiver<FpsSample>,
    events_rx: flume::Receiver<Vec<u8>>,

    on_init: &'a (dyn Fn(ClientInfo) + Send + Sync),
    on_datagram: &'a (dyn Fn(&String, Bytes) + Send + Sync),
    on_rpc: &'a (dyn Fn(&String, u32, SendStream, RecvStream) + Send + Sync),
    on_disconnect: &'a (dyn Fn(&Option<String>) + Send + Sync),
    user_id: Option<String>,
}

impl<'a> Drop for ClientInstance<'a> {
    fn drop(&mut self) {
        log::debug!("Closed server-side connection for {:?}", self.user_id);
        tokio::task::block_in_place(|| {
            (self.on_disconnect)(&self.user_id);
        })
    }
}

impl<'a> ClientInstance<'a> {
    #[tracing::instrument(skip_all)]
    pub async fn run(mut self, conn: NewConnection, server_info: ServerInfo) -> Result<(), NetworkError> {
        log::debug!("Connecting to client");
        let mut proto = ServerProtocol::new(conn, server_info).await?;

        log::debug!("Client loop starting");
        let mut entities_rx = self.diffs_rx.stream();
        let mut stats_rx = self.stats_rx.stream();
        let mut events_rx = self.events_rx.stream();

        tokio::task::block_in_place(|| {
            (self.on_init)(proto.client_info().clone());
        });
        let user_id = proto.client_info().user_id.clone();
        self.user_id = Some(user_id.clone());

        loop {
            tokio::select! {
                Some(msg) = entities_rx.next() => {
                    let span = tracing::debug_span!("world diff");
                    proto.diff_stream.send_bytes(msg).instrument(span).await?;
                }
                Some(msg) = stats_rx.next() => {
                    let span =tracing::debug_span!("stats");
                    proto.stat_stream.send(&msg).instrument(span).await?;
                }

                Some(msg) = events_rx.next() => {
                    let span =tracing::debug_span!("server_event");
                    let mut stream = proto.connection().open_uni().instrument(span).await?;

                    stream.write(&msg).await?;
                }
                Some(Ok(datagram)) = proto.conn.datagrams.next() => {
                    let _span =tracing::debug_span!("datagram").entered();
                    tokio::task::block_in_place(|| (self.on_datagram)(&user_id, datagram))
                }
                Some(Ok((tx, mut rx))) = proto.conn.bi_streams.next() => {
                    let span = tracing::debug_span!("rpc");
                    let stream_id = rx.read_u32().instrument(span).await;
                    if let Ok(stream_id) = stream_id {
                        // tracing::debug!("Read stream id: {stream_id}");
                        tokio::task::block_in_place(|| { (self.on_rpc)(&user_id, stream_id, tx, rx); })
                    }
                }
            }
        }
    }
}

/// Miscellaneous information about the server that needs to be sent to the client during the handshake.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInfo {
    /// The name of the project. Used by the client to figure out what to title its window. Defaults to "Ambient".
    pub project_name: String,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self { project_name: "Ambient".into() }
    }
}
