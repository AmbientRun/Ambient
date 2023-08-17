use ambient_sys::{
    task::RuntimeHandle,
    time::{Instant, SystemTime},
};
use chrono::{DateTime, Utc};
use hierarchy::despawn_recursive;
use std::{sync::Arc, time::Duration};

use ambient_ecs::{
    components, parent, query, Debuggable, Description, DynSystem, Entity, FrameEvent, Name,
    Networked, Resource, Store, System, World,
};
use ambient_gpu::{gpu::Gpu, mesh_buffer::GpuMesh};

use ambient_native_std::asset_cache::{AssetCache, SyncAssetKey};

use serde::{Deserialize, Serialize};

pub mod async_ecs;
pub mod bounding;
pub mod camera;

pub mod hierarchy;
pub mod player;
pub mod transform;
pub mod window;

pub use ambient_ecs::generated::app::components::{
    delta_time, description, epoch_time, game_time, main_scene, map_seed, name, package_name,
    ref_count, selectable, snap_to_ground, tags, ui_scene,
};

/// The time between fixed updates of the server state.
pub const FIXED_SERVER_TICK_TIME: Duration = Duration::from_micros((1_000_000. / 60.) as u64);

components!("app", {
    @[Resource]
    runtime: RuntimeHandle,
    @[Resource]
    gpu: Arc<Gpu>,
    @[Debuggable]
    mesh: Arc<GpuMesh>,

    @[Resource]
    asset_cache: AssetCache,
    @[
        Debuggable, Networked, Store, Resource,
        Name["Session start time"],
        Description["When the current server session was started."]
    ]
    session_start: DateTime<Utc>,
    @[Debuggable, Networked, Store]
    game_mode: GameMode,

    @[Resource, Debuggable]
    app_start_time: Instant,
    @[Resource, Debuggable]
    last_frame_time: Instant,
    @[Resource, Debuggable]
    frame_index: usize,
    @[Debuggable, Store]
    remove_at_game_time: Duration,

    /// Generic component that indicates the entity shouldn't be sent over network
    @[Debuggable, Networked, Store]
    no_sync: (),
});

pub fn init_all_components() {
    init_components();
    window::init_components();
    async_ecs::init_components();
    ambient_gpu_ecs::init_components();
    camera::init_components();
    transform::init_components();
    transform::init_gpu_components();
    bounding::init_components();
    bounding::init_gpu_components();
}

#[derive(Debug, Clone)]
pub struct RuntimeKey;
impl SyncAssetKey<RuntimeHandle> for RuntimeKey {}

#[derive(Debug, Clone)]
pub struct WindowKey;

#[cfg(not(target_os = "unknown"))]
impl SyncAssetKey<Arc<winit::window::Window>> for WindowKey {}

pub fn remove_at_time_system() -> DynSystem {
    query((remove_at_game_time(),)).to_system(|q, world, qs, _| {
        let game_time = *world.resource(self::game_time());
        for (id, (remove_at_time,)) in q.collect_cloned(world, qs) {
            if game_time >= remove_at_time {
                world.despawn(id);
            }
        }
    })
}
pub fn refcount_system() -> DynSystem {
    query(ref_count().changed())
        .excl(parent())
        .to_system(|q, world, qs, _| {
            for (id, count) in q.collect_cloned(world, qs) {
                if count == 0 {
                    despawn_recursive(world, id);
                }
            }
        })
}

/// Returns all the time-related components that need to be
/// created at startup time.
pub fn time_resources_start(delta_time: Duration) -> Entity {
    let system_now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let instant_now = Instant::now();

    Entity::new()
        .with(self::app_start_time(), instant_now)
        .with(self::epoch_time(), system_now)
        .with(self::game_time(), Duration::ZERO)
        .with(self::last_frame_time(), instant_now)
        .with(self::delta_time(), delta_time.as_secs_f32())
}

// Returns all the time-related components that update every frame.
pub fn time_resources_frame(
    frame_time: Instant,
    app_start_time: Instant,
    delta_time: Duration,
) -> Entity {
    let epoch_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    Entity::new()
        .with(self::last_frame_time(), frame_time)
        .with(self::epoch_time(), epoch_time)
        .with(self::game_time(), frame_time - app_start_time)
        .with(self::delta_time(), delta_time.as_secs_f32())
}

#[derive(Debug)]
pub struct FixedTimestepSystem {
    system: DynSystem,
    acc: f32,
    timestep: f32,
}
impl FixedTimestepSystem {
    pub fn new(timestep: f32, system: DynSystem) -> Self {
        Self {
            system,
            timestep,
            acc: 0.,
        }
    }
}
impl System for FixedTimestepSystem {
    #[profiling::function]
    fn run(&mut self, world: &mut World, event: &FrameEvent) {
        let delta_time = *world.resource(self::delta_time());
        self.acc += delta_time;
        while self.acc >= self.timestep {
            self.acc -= self.timestep;
            self.system.run(world, event);
        }
    }
}

#[derive(Debug)]
pub struct ClientTimeResourcesSystem {
    frame_time: Instant,
}
impl ClientTimeResourcesSystem {
    pub fn new() -> Self {
        Self {
            frame_time: Instant::now(),
        }
    }
}
impl System for ClientTimeResourcesSystem {
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        let delta_time = self.frame_time.elapsed();
        self.frame_time = Instant::now();

        world
            .set_components(
                world.resource_entity(),
                time_resources_frame(
                    self.frame_time,
                    *world.resource(self::app_start_time()),
                    delta_time,
                ),
            )
            .unwrap();

        world
            .set(
                world.resource_entity(),
                frame_index(),
                world.resource(frame_index()) + 1,
            )
            .unwrap();
    }
}

#[derive(
    Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Default,
)]
pub enum GameMode {
    #[default]
    Edit,
    Play,
}
