use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Duration};

use crate::{
    client::{ClientConnection, DynRecv, DynSend},
    proto::server::Player,
    NetworkError, RPC_BISTREAM_ID,
};
use ambient_core::{
    name,
    player::{get_by_user_id, player},
};
use ambient_ecs::{
    components, dont_store, query, ArchetypeFilter, Entity, EntityId, FrameEvent, Networked,
    Resource, System, SystemGroup, World, WorldStream, WorldStreamFilter,
};
use ambient_rpc::RpcRegistry;
use ambient_std::{
    asset_cache::AssetCache, asset_url::AbsAssetUrl, fps_counter::FpsSample, log_result,
};
use ambient_sys::time::SystemTime;
use bytes::Bytes;
use flume::Sender;
use parking_lot::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    player_connection: Arc<dyn ClientConnection>,
    // synced resource
    @[Networked]
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
    conn: Arc<dyn ClientConnection>,
    user_id: String,
    entities_tx: Sender<Bytes>,
    connection_id: Uuid,
) -> Entity {
    Entity::new()
        .with(name(), format!("Player {}", user_id))
        .with(ambient_core::player::player(), ())
        .with(ambient_core::player::user_id(), user_id)
        .with(player_connection(), conn)
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
                ambient_sys::task::spawn(async move {
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
            .set(self.world.resource_entity(), ambient_core::abs_time(), time)
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
