use std::{sync::Arc, time::Duration};

pub use ambient_core::gpu;
use ambient_core::{
    app_start_time, asset_cache,
    async_ecs::async_ecs_systems,
    bounding::bounding_systems,
    camera::camera_systems,
    frame_index,
    gpu_ecs::{gpu_world, GpuWorld, GpuWorldSyncEvent, GpuWorldUpdate},
    hierarchy::dump_world_hierarchy_to_tmp_file,
    name, remove_at_time_system, runtime, time,
    transform::TransformSystem,
    window::WindowCtl,
    TimeResourcesSystem,
};
use ambient_ecs::{
    components, world_events, Debuggable, Entity, FrameEvent, MakeDefault, MaybeResource, System,
    SystemGroup, World, WorldEventsSystem,
};
use ambient_element::ambient_system;
use ambient_gizmos::{gizmos, Gizmos};
use ambient_gpu::{gpu::Gpu, mesh_buffer::MeshBufferKey};
use ambient_renderer::lod::lod_system;
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    fps_counter::FpsSample,
};
use ambient_sys::{task::RuntimeHandle, time::SystemTime};
use glam::{UVec2, Vec2};
use renderers::examples_renderer;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

mod app;
mod renderers;
mod window;
pub use app::*;
pub use window::*;

fn default_title() -> String {
    "ambient".into()
}

components!("app", {
    @[MakeDefault[default_title], Debuggable, MaybeResource]
    window_title: String,
    fps_stats: FpsSample,
});

pub fn init_all_components() {
    ambient_ecs::init_components();
    ambient_core::init_all_components();
    ambient_element::init_components();
    ambient_animation::init_components();
    ambient_gizmos::init_components();
    ambient_cameras::init_all_components();
    init_components();
    ambient_renderer::init_all_components();
    ambient_ui_native::init_all_components();
    ambient_input::init_all_components();
    ambient_model::init_components();
    ambient_cameras::init_all_components();
    renderers::init_components();
}

pub fn gpu_world_sync_systems() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "gpu_world",
        vec![
            // Note: All Gpu sync systems must run immediately after GpuWorldUpdate, as that's the only time we know
            // the layout of the GpuWorld is correct
            Box::new(GpuWorldUpdate),
            Box::new(ambient_core::transform::transform_gpu_systems()),
            Box::new(ambient_renderer::gpu_world_systems()),
            Box::new(ambient_core::bounding::gpu_world_systems()),
            Box::new(ambient_ui_native::layout::gpu_world_systems()),
        ],
    )
}

pub fn world_instance_systems(full: bool) -> SystemGroup {
    SystemGroup::new(
        "world_instance",
        vec![
            Box::new(TimeResourcesSystem::new()),
            Box::new(async_ecs_systems()),
            remove_at_time_system(),
            Box::new(WorldEventsSystem),
            if full {
                Box::new(ambient_input::picking::frame_systems())
            } else {
                Box::new(DummySystem)
            },
            Box::new(lod_system()),
            Box::new(ambient_renderer::systems()),
            Box::new(ambient_system()),
            if full {
                Box::new(ambient_ui_native::systems())
            } else {
                Box::new(DummySystem)
            },
            Box::new(ambient_model::model_systems()),
            Box::new(ambient_animation::animation_systems()),
            Box::new(TransformSystem::new()),
            Box::new(ambient_renderer::skinning::skinning_systems()),
            Box::new(bounding_systems()),
            Box::new(camera_systems()),
        ],
    )
}

pub struct AppResources {
    pub assets: AssetCache,
    pub gpu: Arc<Gpu>,
    pub runtime: RuntimeHandle,
    pub ctl_tx: flume::Sender<WindowCtl>,
    window_physical_size: UVec2,
    window_logical_size: UVec2,
    window_scale_factor: f64,
}

impl AppResources {
    pub fn from_world(world: &World) -> Self {
        Self {
            assets: world.resource(self::asset_cache()).clone(),
            gpu: world.resource(self::gpu()).clone(),
            runtime: world.resource(self::runtime()).clone(),
            ctl_tx: world.resource(ambient_core::window::window_ctl()).clone(),
            window_physical_size: *world.resource(ambient_core::window::window_physical_size()),
            window_logical_size: *world.resource(ambient_core::window::window_logical_size()),
            window_scale_factor: *world.resource(ambient_core::window::window_scale_factor()),
        }
    }
}

pub fn world_instance_resources(resources: AppResources) -> Entity {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    Entity::new()
        .with(name(), "Resources".to_string())
        .with(self::gpu(), resources.gpu.clone())
        .with(gizmos(), Gizmos::new())
        .with(self::runtime(), resources.runtime)
        .with(self::window_title(), "".to_string())
        .with(self::fps_stats(), FpsSample::default())
        .with(self::asset_cache(), resources.assets.clone())
        .with_default(world_events())
        .with(frame_index(), 0_usize)
        .with(ambient_core::window::cursor_position(), Vec2::ZERO)
        .with(ambient_core::app_start_time(), current_time)
        .with(ambient_core::time(), current_time)
        .with(ambient_core::dtime(), 0.)
        .with(gpu_world(), GpuWorld::new_arced(resources.assets))
        .with_merge(ambient_input::resources())
        .with_merge(ambient_input::picking::resources())
        .with_merge(ambient_core::async_ecs::async_ecs_resources())
        .with(
            ambient_core::window::window_physical_size(),
            resources.window_physical_size,
        )
        .with(
            ambient_core::window::window_logical_size(),
            resources.window_logical_size,
        )
        .with(
            ambient_core::window::window_scale_factor(),
            resources.window_scale_factor,
        )
        .with(ambient_core::window::window_ctl(), resources.ctl_tx)
}

pub fn get_time_since_app_start(world: &World) -> Duration {
    *world.resource(time()) - *world.resource(app_start_time())
}

#[derive(Debug)]
pub struct ExamplesSystem;
impl System<Event<'static, ()>> for ExamplesSystem {
    #[allow(clippy::single_match)]
    fn run(&mut self, world: &mut World, event: &Event<'static, ()>) {
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(virtual_keycode),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => match virtual_keycode {
                VirtualKeyCode::F1 => dump_world_hierarchy_to_tmp_file(world),
                VirtualKeyCode::F2 => world.dump_to_tmp_file(),
                VirtualKeyCode::F3 => world
                    .resource(examples_renderer())
                    .lock()
                    .dump_to_tmp_file(),
                _ => {}
            },
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct MeshBufferUpdate;
impl System for MeshBufferUpdate {
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        ambient_profiling::scope!("MeshBufferUpdate.run");
        let assets = world.resource(asset_cache()).clone();
        let mesh_buffer = MeshBufferKey.get(&assets);
        let mut mesh_buffer = mesh_buffer.lock();
        mesh_buffer.update();
    }
}

#[derive(Debug)]
pub struct DummySystem;
impl System for DummySystem {
    fn run(&mut self, _world: &mut World, _event: &FrameEvent) {}
}
