#[macro_use]
extern crate lazy_static;

use std::{
    sync::Arc, time::{Duration, Instant, SystemTime}
};

use elements_ecs::{components, query, DynSystem, EntityId, FrameEvent, QueryState, System, World};
use elements_gpu::{gpu::Gpu, mesh_buffer::GpuMesh};

pub mod async_ecs;
pub mod gpu_ecs;
pub mod hierarchy;
use elements_std::{
    asset_cache::{AssetCache, SyncAssetKey}, events::EventDispatcher, math::interpolate
};
use glam::{uvec2, vec2, UVec2, Vec2};
pub use paste;
use winit::{event::Event, window::Window};
pub mod bounding;
pub mod camera;
pub mod transform;

components!("app", {
    name: String,
    runtime: tokio::runtime::Handle,
    gpu: Arc<Gpu>,
    mesh: Arc<GpuMesh>,
    window: Arc<Window>,
    window_scale_factor: f64,
    /// The logical size is the physical size divided by the scale factor
    window_logical_size: UVec2,
    /// The physical size is the actual number of pixels on the screen
    window_physical_size: UVec2,
    /// Mouse position in screen space
    mouse_position: Vec2,
    main_scene: (),
    ui_scene: (),
    asset_cache: AssetCache,

    time: Duration,
    dtime: f32,
    app_start_time: Duration,
    frame_index: usize,
    remove_at_time: Duration,

    on_frame: EventDispatcher<dyn Fn(&mut World, EntityId, f32) + Sync + Send>,

    on_event: EventDispatcher<dyn Fn(&mut World, EntityId, &winit::event::Event<()>) + Sync + Send>,
    on_window_event: EventDispatcher<dyn Fn(&mut World, EntityId, &winit::event::WindowEvent) + Sync + Send>,
    on_device_event: EventDispatcher<dyn Fn(&mut World, EntityId, &winit::event::DeviceEvent) + Sync + Send>,

    /// Generic component that indicates the entity shouldn't be sent over network
    no_sync: (),
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
impl SyncAssetKey<tokio::runtime::Handle> for RuntimeKey {}

#[derive(Debug, Clone)]
pub struct WindowKey;
impl SyncAssetKey<Arc<Window>> for WindowKey {}

#[derive(Debug)]
pub struct WinitEventsSystem {
    event_qs: QueryState,
    window_event_qs: QueryState,
    device_event_qs: QueryState,
}
impl WinitEventsSystem {
    pub fn new() -> Self {
        Self { event_qs: QueryState::new(), window_event_qs: QueryState::new(), device_event_qs: QueryState::new() }
    }
}
impl System<Event<'static, ()>> for WinitEventsSystem {
    fn run(&mut self, world: &mut World, event: &Event<'static, ()>) {
        for (id, (dispatcher,)) in query((on_event(),)).collect_cloned(world, Some(&mut self.event_qs)) {
            for handler in dispatcher.iter() {
                handler(world, id, event);
            }
        }
        match event {
            Event::WindowEvent { event, .. } => {
                for (id, (dispatcher,)) in query((on_window_event(),)).collect_cloned(world, Some(&mut self.window_event_qs)) {
                    for handler in dispatcher.iter() {
                        handler(world, id, event);
                    }
                }
            }
            Event::DeviceEvent { ref event, .. } => {
                for (id, (dispatcher,)) in query((on_device_event(),)).collect_cloned(world, Some(&mut self.device_event_qs)) {
                    for handler in dispatcher.iter() {
                        handler(world, id, event);
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn on_frame_system() -> DynSystem {
    query((on_frame(),)).to_system(|q, world, qs, _| {
        let dtime = *world.resource(self::dtime());
        for (id, (on_frame,)) in q.collect_cloned(world, qs) {
            for handler in on_frame.iter() {
                handler(world, id, dtime);
            }
        }
    })
}

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

/// Updates `window_physical_size`, `window_logical_size` and `window_scale_factor` from the `window` component
#[derive(Debug)]
pub struct WindowSyncSystem;
impl System for WindowSyncSystem {
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        if let Some(window) = world.resource_opt(window()).cloned() {
            let size = uvec2(window.inner_size().width, window.inner_size().height);
            world.set_if_changed(world.resource_entity(), self::window_physical_size(), size).unwrap();
            world
                .set_if_changed(world.resource_entity(), self::window_logical_size(), (size.as_dvec2() / window.scale_factor()).as_uvec2())
                .unwrap();
            world.set_if_changed(world.resource_entity(), self::window_scale_factor(), window.scale_factor()).unwrap();
        }
    }
}
