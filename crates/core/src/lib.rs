#[macro_use]
extern crate lazy_static;

use ambient_sys::{task::RuntimeHandle, time::Instant, time::SystemTime};
use chrono::{DateTime, Utc};
use std::{sync::Arc, time::Duration};
use window::WindowCtl;

use ambient_ecs::{components, query, Debuggable, Description, DynSystem, FrameEvent, Name, Networked, Resource, Store, System, World};
use ambient_gpu::{gpu::Gpu, mesh_buffer::GpuMesh};

use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey},
    math::interpolate,
};
use glam::{uvec2, vec2, UVec2, Vec2};
pub use paste;
use serde::{Deserialize, Serialize};
use winit::window::Window;
pub mod async_ecs;
pub mod bounding;
pub mod camera;
pub mod gpu_ecs;
pub mod hierarchy;
pub mod transform;
pub mod window;

components!("app", {
    @[Debuggable, Networked, Store, Name["Name"], Description["A human-friendly name for this entity."]]
    name: String,
    @[Resource]
    runtime: RuntimeHandle,
    @[Resource]
    gpu: Arc<Gpu>,
    @[Debuggable]
    mesh: Arc<GpuMesh>,

    @[Resource, Name["Window Control"], Description["Allows controlling the window from afar."]]
    window_ctl: flume::Sender<WindowCtl>,

    @[Resource, Name["Window scale factor"], Description["This number is usually 1, but on for instance retina displays it's 2."]]
    window_scale_factor: f64,
    @[Resource, Name["Window logical size"], Description["The logical size is the physical size divided by the scale factor."]]
    window_logical_size: UVec2,
    @[Resource, Name["Window physical size"], Description["The physical size is the actual number of pixels on the screen."]]
    window_physical_size: UVec2,
    /// Mouse position in screen space
    @[Resource]
    mouse_position: Vec2,
    @[
        Debuggable, Networked, Store,
        Name["Main scene"],
        Description["If attached, this entity belongs to the main scene."]
    ]
    main_scene: (),
    @[
        Debuggable, Networked, Store,
        Name["UI scene"],
        Description["If attached, this entity belongs to the UI scene."]
    ]
    ui_scene: (),
    @[Resource]
    asset_cache: AssetCache,
    @[
        Debuggable, Networked, Store,
        Name["Map seed"],
        Description["A random number seed for this map."]
    ]
    map_seed: u64,
    @[
        Debuggable, Networked, Store,
        Name["Snap to ground"],
        Description["This object should automatically be moved with the terrain if the terrain is changed.\nThe value is the offset from the terrain."]
    ]
    snap_to_ground: f32,
    @[
        Debuggable, Networked, Store,
        Name["Selectable"],
        Description["If attached, this object can be selected in the editor."]
    ]
    selectable: (),
    @[
        Debuggable, Networked, Store, Resource,
        Name["Session start time"],
        Description["When the current server session was started."]
    ]
    session_start: DateTime<Utc>,
    @[
        Debuggable, Networked, Store,
        Name["Tags"],
        Description["Tags for categorizing this entity."]
    ]
    tags: Vec<String>,
    @[Debuggable, Networked, Store]
    game_mode: GameMode,

    @[Resource]
    time: Duration,
    @[Resource, Debuggable, Name["Delta Time"], Description["How long the previous tick took in seconds.\nAlso known as frametime."]]
    dtime: f32,
    @[Resource]
    app_start_time: Duration,
    @[Resource]
    frame_index: usize,
    @[Debuggable, Store]
    remove_at_time: Duration,

    /// Generic component that indicates the entity shouldn't be sent over network
    @[Debuggable, Networked, Store]
    no_sync: (),

    @[
        Resource, Debuggable,
        Name["Project Name"],
        Description["The name of the project, from the manifest.\nDefaults to \"Ambient\"."]
    ]
    project_name: String,
});

pub fn init_all_components() {
    init_components();
    hierarchy::init_components();
    async_ecs::init_components();
    gpu_ecs::init_components();
    camera::init_components();
    transform::init_components();
    transform::init_gpu_components();
    bounding::init_components();
    bounding::init_gpu_components();
}

pub fn screen_to_clip_space(world: &World, screen_pos: Vec2) -> Vec2 {
    let screen_size = *world.resource(window_physical_size());
    interpolate(screen_pos, Vec2::ZERO, screen_size.as_vec2(), vec2(-1., 1.), vec2(1., -1.))
}
pub fn get_mouse_clip_space_position(world: &World) -> Vec2 {
    let mouse_position = *world.resource(mouse_position());
    screen_to_clip_space(world, mouse_position)
}

#[derive(Debug, Clone)]
pub struct RuntimeKey;
impl SyncAssetKey<RuntimeHandle> for RuntimeKey {}

#[derive(Debug, Clone)]
pub struct WindowKey;
#[cfg(not(target_os = "unknown"))]
impl SyncAssetKey<Arc<Window>> for WindowKey {}

pub fn remove_at_time_system() -> DynSystem {
    query((remove_at_time(),)).to_system(|q, world, qs, _| {
        let time = *world.resource(self::time());
        for (id, (remove_at_time,)) in q.collect_cloned(world, qs) {
            if time >= remove_at_time {
                world.despawn(id);
            }
        }
    })
}

#[derive(Debug)]
pub struct FixedTimestepSystem {
    system: DynSystem,
    acc: f32,
    timestep: f32,
}
impl FixedTimestepSystem {
    pub fn new(timestep: f32, system: DynSystem) -> Self {
        Self { system, timestep, acc: 0. }
    }
}
impl System for FixedTimestepSystem {
    #[profiling::function]
    fn run(&mut self, world: &mut World, event: &FrameEvent) {
        let dtime = *world.resource(self::dtime());
        self.acc += dtime;
        while self.acc >= self.timestep {
            self.acc -= self.timestep;
            self.system.run(world, event);
        }
    }
}

#[derive(Debug)]
pub struct TimeResourcesSystem {
    frame_time: Instant,
}
impl TimeResourcesSystem {
    pub fn new() -> Self {
        Self { frame_time: Instant::now() }
    }
}
impl System for TimeResourcesSystem {
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        let dtime = self.frame_time.elapsed().as_secs_f32();
        self.frame_time = Instant::now();
        let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        world.set(world.resource_entity(), self::time(), time).unwrap();
        world.set(world.resource_entity(), self::dtime(), dtime).unwrap();
        world.set(world.resource_entity(), frame_index(), world.resource(frame_index()) + 1).unwrap();
    }
}

pub fn get_window_sizes(window: &Window) -> (UVec2, UVec2, f64) {
    let size = uvec2(window.inner_size().width, window.inner_size().height);
    let sf = window.scale_factor();
    (size, (size.as_dvec2() / sf).as_uvec2(), sf)
}

pub fn mirror_window_components(src: &mut World, dst: &mut World) {
    let dr = dst.resource_entity();

    dst.set_if_changed(dr, window_physical_size(), *src.resource(window_physical_size())).unwrap();
    dst.set_if_changed(dr, window_logical_size(), *src.resource(window_logical_size())).unwrap();
    dst.set_if_changed(dr, window_scale_factor(), *src.resource(window_scale_factor())).unwrap();

    dst.set_if_changed(dr, mouse_position(), *src.resource(mouse_position())).unwrap();
}

/// Updates `window_physical_size`, `window_logical_size` and `window_scale_factor` from the `window` component
// #[derive(Debug)]
// pub struct WindowSyncSystem;
// impl System for WindowSyncSystem {
//     fn run(&mut self, world: &mut World, _event: &FrameEvent) {
//         if let Some(window) = world.resource_opt(window()).cloned() {
//             let size = uvec2(window.inner_size().width, window.inner_size().height);
//             world.set_if_changed(world.resource_entity(), self::window_physical_size(), size).unwrap();
//             world
//                 .set_if_changed(world.resource_entity(), self::window_logical_size(), (size.as_dvec2() / window.scale_factor()).as_uvec2())
//                 .unwrap();
//             world.set_if_changed(world.resource_entity(), self::window_scale_factor(), window.scale_factor()).unwrap();
//         }
//     }
// }

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Default)]
pub enum GameMode {
    #[default]
    Edit,
    Play,
}
