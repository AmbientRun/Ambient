use std::{
    sync::Arc, time::{Duration, SystemTime}
};

use elements_cameras::assets_camera_systems;
pub use elements_core::gpu;
use elements_core::{
    app_start_time, asset_cache, async_ecs::async_ecs_systems, bounding::bounding_systems, camera::camera_systems, frame_index, gpu_ecs::{gpu_world, GpuWorld, GpuWorldSyncEvent, GpuWorldUpdate}, hierarchy::dump_world_hierarchy_to_tmp_file, mouse_position, on_frame_system, remove_at_time_system, runtime, time, transform::TransformSystem, window_scale_factor, RuntimeKey, TimeResourcesSystem, WindowKey, WindowSyncSystem, WinitEventsSystem
};
use elements_ecs::{components, ComponentRegistry, DynSystem, EntityData, FrameEvent, System, SystemGroup, World};
use elements_element::elements_system;
use elements_gizmos::{gizmos, Gizmos};
use elements_gpu::{
    gpu::{Gpu, GpuKey}, mesh_buffer::MeshBufferKey
};
use elements_renderer::lod::lod_system;
use elements_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt}, fps_counter::{FpsCounter, FpsSample}
};
use glam::{uvec2, vec2, Vec2};
use parking_lot::Mutex;
use renderers::{examples_renderer, ui_renderer, UIRender};
use tokio::runtime::Runtime;
use winit::{
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder}
};

use crate::renderers::ExamplesRender;

mod renderers;

components!("app", {
    window_title: String,
    fps_stats: FpsSample,
});

pub fn init_all_components() {
    elements_ecs::init_components();
    elements_core::init_all_components();
    elements_element::init_components();
    elements_animation::init_components();
    elements_gizmos::init_components();
    elements_cameras::init_all_components();
    init_components();
    elements_renderer::init_all_componets();
    elements_ui::init_all_componets();
    elements_input::init_all_components();
    elements_model::init_components();
    elements_cameras::init_all_components();
    renderers::init_components();
}

pub fn gpu_world_sync_systems() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "gpu_world",
        vec![
            // Note: All Gpu sync systems must run immediately after GpuWorldUpdate, as that's the only time we know
            // the layout of the GpuWorld is correct
            Box::new(GpuWorldUpdate),
            Box::new(elements_core::transform::transform_gpu_systems()),
            Box::new(elements_renderer::gpu_world_systems()),
            Box::new(elements_core::bounding::gpu_world_systems()),
            Box::new(elements_ui::layout::gpu_world_systems()),
        ],
    )
}

pub fn world_instance_systems(full: bool) -> SystemGroup {
    SystemGroup::new(
        "world_instance",
        vec![
            Box::new(TimeResourcesSystem::new()),
            Box::new(async_ecs_systems()),
            on_frame_system(),
            remove_at_time_system(),
            Box::new(WindowSyncSystem),
            if full { Box::new(elements_input::picking::frame_systems()) } else { Box::new(DummySystem) },
            Box::new(lod_system()),
            Box::new(elements_renderer::systems()),
            Box::new(elements_system()),
            if full { Box::new(elements_ui::systems()) } else { Box::new(DummySystem) },
            Box::new(elements_model::model_systems()),
            Box::new(elements_animation::animation_systems()),
            Box::new(TransformSystem::new()),
            Box::new(elements_renderer::skinning::skinning_systems()),
            Box::new(bounding_systems()),
            Box::new(camera_systems()),
        ],
    )
}
pub struct AppResources {
    pub assets: AssetCache,
    pub gpu: Arc<Gpu>,
    pub runtime: tokio::runtime::Handle,
    pub window: Option<Arc<Window>>,
}
impl AppResources {
    pub fn from_world(world: &World) -> Self {
        Self {
            assets: world.resource(self::asset_cache()).clone(),
            gpu: world.resource(self::gpu()).clone(),
            runtime: world.resource(self::runtime()).clone(),
            window: Some(world.resource(elements_core::window()).clone()),
        }
    }
    pub fn from_assets(assets: &AssetCache) -> Self {
        Self { assets: assets.clone(), gpu: GpuKey.get(assets), runtime: RuntimeKey.get(assets), window: WindowKey.try_get(assets) }
    }
}
pub fn world_instance_resources(resources: AppResources) -> EntityData {
    let mut ed = EntityData::new()
        .set(self::gpu(), resources.gpu.clone())
        .set(gizmos(), Gizmos::new())
        .set(self::runtime(), resources.runtime)
        .set(self::window_title(), "".to_string())
        .set(self::fps_stats(), FpsSample::default())
        .set(self::asset_cache(), resources.assets.clone())
        .set(frame_index(), 0_usize)
        .set(elements_core::mouse_position(), Vec2::ZERO)
        .set(elements_core::app_start_time(), SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap())
        .set(elements_core::time(), SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap())
        .set(elements_core::dtime(), 0.)
        .set(elements_core::window_logical_size(), uvec2(100, 100))
        .set(elements_core::window_physical_size(), uvec2(100, 100))
        .set(gpu_world(), GpuWorld::new_arced(resources.assets.clone()))
        .append(elements_input::picking::resources())
        .append(elements_core::async_ecs::async_ecs_resources());
    if let Some(window) = resources.window {
        ed = ed.set(elements_core::window(), window.clone()).set(elements_core::window_scale_factor(), window.scale_factor());
    }
    ed
}
pub fn get_time_since_app_start(world: &World) -> Duration {
    *world.resource(time()) - *world.resource(app_start_time())
}

pub struct AppBuilder {
    pub event_loop: Option<EventLoop<()>>,
    pub runtime: Option<Runtime>,
    pub window_builder: Option<WindowBuilder>,
    pub asset_cache: Option<AssetCache>,
    pub ui_renderer: bool,
    pub main_renderer: bool,
}
impl AppBuilder {
    pub fn new() -> Self {
        Self { event_loop: None, runtime: None, window_builder: None, asset_cache: None, ui_renderer: false, main_renderer: true }
    }
    pub fn simple() -> Self {
        Self::new()
    }
    pub fn simple_ui() -> Self {
        Self::new().ui_renderer(true).main_renderer(false)
    }
    pub fn simple_dual() -> Self {
        Self::new().ui_renderer(true).main_renderer(true)
    }
    pub fn with_event_loop(mut self, event_loop: EventLoop<()>) -> Self {
        self.event_loop = Some(event_loop);
        self
    }
    pub fn with_runtime(mut self, runtime: Runtime) -> Self {
        self.runtime = Some(runtime);
        self
    }
    pub fn with_window_builder(mut self, window_builder: WindowBuilder) -> Self {
        self.window_builder = Some(window_builder);
        self
    }
    pub fn with_asset_cache(mut self, asset_cache: AssetCache) -> Self {
        self.asset_cache = Some(asset_cache);
        self
    }
    pub fn ui_renderer(mut self, value: bool) -> Self {
        self.ui_renderer = value;
        self
    }
    pub fn main_renderer(mut self, value: bool) -> Self {
        self.main_renderer = value;
        self
    }
    pub fn build(self) -> anyhow::Result<App> {
        crate::init_all_components();
        let event_loop = self.event_loop.unwrap_or_else(|| EventLoop::new());
        let window = self.window_builder.unwrap_or_default();
        let window = Arc::new(window.build(&event_loop).unwrap());

        #[cfg(feature = "profile")]
        let puffin_server = {
            let puffin_addr = format!(
                "0.0.0.0:{}",
                std::env::var("PUFFIN_PORT").ok().and_then(|port| port.parse::<u16>().ok()).unwrap_or(puffin_http::DEFAULT_PORT)
            );
            let server = puffin_http::Server::new(&puffin_addr)?;
            tracing::info!("Puffin server running on {}", puffin_addr);
            puffin::set_scopes_on(true);
            server
        };

        let _ = thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Max);
        let runtime = self.runtime.unwrap_or_else(|| tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap());

        let mut world = World::new("main_app");
        let gpu = Arc::new(runtime.block_on(async { Gpu::with_config(Some(&window), true).await }));
        let assets = self.asset_cache.unwrap_or_else(|| AssetCache::new(runtime.handle().clone()));
        RuntimeKey.insert(&assets, runtime.handle().clone());
        GpuKey.insert(&assets, gpu.clone());
        WindowKey.insert(&assets, window.clone());

        let app_resources =
            AppResources { gpu: gpu.clone(), runtime: runtime.handle().clone(), assets: assets.clone(), window: Some(window.clone()) };

        let resources = world_instance_resources(app_resources);
        world.add_components(world.resource_entity(), resources).unwrap();
        if self.ui_renderer || self.main_renderer {
            if !self.main_renderer {
                let renderer = Arc::new(Mutex::new(UIRender::new(&mut world)));
                world.add_resource(ui_renderer(), renderer);
            } else {
                let renderer = Arc::new(Mutex::new(ExamplesRender::new(&mut world, self.ui_renderer, self.main_renderer)));
                world.add_resource(examples_renderer(), renderer);
            }
        }

        Ok(App {
            window_focused: true,
            window,
            runtime,
            systems: SystemGroup::new("app", vec![Box::new(MeshBufferUpdate), Box::new(world_instance_systems(true))]),
            world,
            gpu_world_sync_systems: gpu_world_sync_systems(),
            window_event_systems: SystemGroup::new(
                "window_event_systems",
                vec![
                    Box::new(assets_camera_systems()),
                    Box::new(WinitEventsSystem::new()),
                    Box::new(elements_input::event_systems()),
                    Box::new(renderers::systems()),
                ],
            ),
            event_loop: Some(event_loop),

            fps: FpsCounter::new(),
            #[cfg(feature = "profile")]
            _puffin: puffin_server,
            modifiers: Default::default(),
        })
    }
    pub fn run(self, init: impl FnOnce(&mut App, tokio::runtime::Handle)) {
        let mut app = self.build().unwrap();
        let runtime = app.runtime.handle().clone();
        init(&mut app, runtime);
        app.run()
    }
    pub fn run_world(self, init: impl FnOnce(&mut World)) {
        self.run(|app, _| init(&mut app.world))
    }
}

pub struct App {
    pub world: World,
    pub systems: SystemGroup,
    pub gpu_world_sync_systems: SystemGroup<GpuWorldSyncEvent>,
    pub window_event_systems: SystemGroup<Event<'static, ()>>,
    pub runtime: Runtime,
    pub window: Arc<Window>,
    event_loop: Option<EventLoop<()>>,
    fps: FpsCounter,
    #[cfg(feature = "profile")]
    _puffin: puffin_http::Server,
    modifiers: ModifiersState,

    window_focused: bool,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("App");
        d.field("world", &self.world)
            .field("systems", &self.systems)
            .field("gpu_world_sync_systems", &self.gpu_world_sync_systems)
            .field("window_event_systems", &self.window_event_systems)
            .field("runtime", &self.runtime)
            .field("window", &self.window)
            .field("fps", &self.fps)
            .field("window_focused", &self.window_focused);

        #[cfg(feature = "profile")]
        d.field("puffin", &true);
        #[cfg(not(feature = "profile"))]
        d.field("puffin", &false);

        d.finish()
    }
}
impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }
    pub fn run(mut self) {
        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow| {
            // HACK(mithun): treat dpi changes as resize events. ideally we'd handle this in handle_event proper,
            // but https://github.com/rust-windowing/winit/issues/1968 restricts us
            if let Event::WindowEvent { window_id, event: WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } } = &event {
                *self.world.resource_mut(window_scale_factor()) = *scale_factor;
                self.handle_event(
                    &Event::WindowEvent { window_id: *window_id, event: WindowEvent::Resized(**new_inner_size) },
                    control_flow,
                );
            } else if let Some(event) = event.to_static() {
                self.handle_event(&event, control_flow);
            }
        });
    }
    pub fn handle_event(&mut self, event: &Event<'static, ()>, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Poll;

        // From: https://github.com/gfx-rs/wgpu/issues/1783
        // TODO: According to the issue we should cap the framerate instead
        #[cfg(target_os = "macos")]
        if !self.window_focused {
            *control_flow = ControlFlow::Wait;
        }

        let world = &mut self.world;
        let systems = &mut self.systems;
        let gpu_world_sync_systems = &mut self.gpu_world_sync_systems;
        world.resource(gpu()).device.poll(wgpu::Maintain::Poll);
        self.window_event_systems.run(world, event);
        match event {
            Event::MainEventsCleared => {
                profiling::scope!("frame");
                world.next_frame();
                {
                    profiling::scope!("systems");
                    systems.run(world, &FrameEvent);
                    gpu_world_sync_systems.run(world, &GpuWorldSyncEvent);
                }

                if let Some(fps) = self.fps.frame_next() {
                    world.set(world.resource_entity(), self::fps_stats(), fps.clone()).unwrap();
                    self.window.set_title(&format!("{} [{}, {} entities]", world.resource(window_title()), fps.dump_both(), world.len()));
                }
                self.window.request_redraw();
                profiling::finish_frame!();
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Focused(focused) => {
                    self.window_focused = *focused;
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    *self.world.resource_mut(window_scale_factor()) = *scale_factor;
                }
                WindowEvent::Resized(size) => {
                    let gpu = world.resource(gpu()).clone();
                    gpu.resize(*size);
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            if let VirtualKeyCode::Q = keycode {
                                if self.modifiers.logo() {
                                    *control_flow = ControlFlow::Exit;
                                }
                            }
                        }
                    }
                }
                WindowEvent::ModifiersChanged(state) => {
                    self.modifiers = *state;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if self.window_focused {
                        world.set(world.resource_entity(), mouse_position(), vec2(position.x as f32, position.y as f32)).unwrap();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    pub fn add_system(&mut self, system: DynSystem) -> &mut Self {
        self.systems.add(system);
        self
    }
}

#[derive(Debug)]
pub struct ExamplesSystem;
impl System<Event<'static, ()>> for ExamplesSystem {
    fn run(&mut self, world: &mut World, event: &Event<'static, ()>) {
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput { virtual_keycode: Some(virtual_keycode), state: ElementState::Pressed, .. },
                        ..
                    },
                ..
            } => match virtual_keycode {
                VirtualKeyCode::F1 => dump_world_hierarchy_to_tmp_file(world),
                VirtualKeyCode::F2 => world.dump_to_tmp_file(),
                VirtualKeyCode::F3 => world.resource(examples_renderer()).lock().dump_to_tmp_file(),
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
        profiling::scope!("MeshBufferUpdate.run");
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
