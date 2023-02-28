use std::{future::Future, sync::Arc, time::Duration};

use ambient_cameras::assets_camera_systems;
pub use ambient_core::gpu;
use ambient_core::{
    app_start_time, asset_cache,
    async_ecs::async_ecs_systems,
    bounding::bounding_systems,
    camera::camera_systems,
    frame_index, get_window_sizes,
    gpu_ecs::{gpu_world, GpuWorld, GpuWorldSyncEvent, GpuWorldUpdate},
    hierarchy::dump_world_hierarchy_to_tmp_file,
    mouse_position, on_frame_system, remove_at_time_system, runtime, time,
    transform::TransformSystem,
    window::WindowCtl,
    window_logical_size, window_physical_size, window_scale_factor, RuntimeKey, TimeResourcesSystem, WinitEventsSystem,
};
use ambient_ecs::{components, Debuggable, DynSystem, EntityData, FrameEvent, MakeDefault, MaybeResource, System, SystemGroup, World};
use ambient_element::ambient_system;
use ambient_gizmos::{gizmos, Gizmos};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    mesh_buffer::MeshBufferKey,
};
use ambient_renderer::lod::lod_system;
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    fps_counter::{FpsCounter, FpsSample},
};
use ambient_sys::task::RuntimeHandle;
use glam::{uvec2, vec2, UVec2, Vec2};
use parking_lot::Mutex;
use renderers::{examples_renderer, ui_renderer, UIRender};
use tracing::info_span;
use winit::{
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::renderers::ExamplesRender;

mod renderers;

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
    ambient_renderer::init_all_componets();
    ambient_ui::init_all_componets();
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
            Box::new(ambient_ui::layout::gpu_world_systems()),
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
            if full { Box::new(ambient_input::picking::frame_systems()) } else { Box::new(DummySystem) },
            Box::new(lod_system()),
            Box::new(ambient_renderer::systems()),
            Box::new(ambient_system()),
            if full { Box::new(ambient_ui::systems()) } else { Box::new(DummySystem) },
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
            ctl_tx: world.resource(ambient_core::window_ctl()).clone(),
            window_physical_size: *world.resource(ambient_core::window_physical_size()),
            window_logical_size: *world.resource(ambient_core::window_logical_size()),
            window_scale_factor: *world.resource(ambient_core::window_scale_factor()),
        }
    }
}

pub fn world_instance_resources(resources: AppResources) -> EntityData {
    let current_time = ambient_sys::time::current_epoch_time();
    EntityData::new()
        .set(self::gpu(), resources.gpu.clone())
        .set(gizmos(), Gizmos::new())
        .set(self::runtime(), resources.runtime)
        .set(self::window_title(), "".to_string())
        .set(self::fps_stats(), FpsSample::default())
        .set(self::asset_cache(), resources.assets.clone())
        .set(frame_index(), 0_usize)
        .set(ambient_core::mouse_position(), Vec2::ZERO)
        .set(ambient_core::app_start_time(), current_time)
        .set(ambient_core::time(), current_time)
        .set(ambient_core::dtime(), 0.)
        .set(gpu_world(), GpuWorld::new_arced(resources.assets))
        .append(ambient_input::picking::resources())
        .append(ambient_core::async_ecs::async_ecs_resources())
        .set(ambient_core::window_physical_size(), resources.window_physical_size)
        .set(ambient_core::window_logical_size(), resources.window_logical_size)
        .set(ambient_core::window_scale_factor(), resources.window_scale_factor)
        .set(ambient_core::window_ctl(), resources.ctl_tx)
}

pub fn get_time_since_app_start(world: &World) -> Duration {
    *world.resource(time()) - *world.resource(app_start_time())
}

pub struct AppBuilder {
    pub event_loop: Option<EventLoop<()>>,
    pub window_builder: Option<WindowBuilder>,
    pub asset_cache: Option<AssetCache>,
    pub ui_renderer: bool,
    pub main_renderer: bool,
    pub examples_systems: bool,
}

pub trait AsyncInit<'a> {
    type Future: 'a + Future<Output = ()>;
    fn call(self, app: &'a mut App) -> Self::Future;
}

impl<'a, F, Fut> AsyncInit<'a> for F
where
    Fut: 'a + Future<Output = ()>,
    F: FnOnce(&'a mut App) -> Fut,
{
    type Future = Fut;

    fn call(self, app: &'a mut App) -> Self::Future {
        (self)(app)
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        Self { event_loop: None, window_builder: None, asset_cache: None, ui_renderer: false, main_renderer: true, examples_systems: false }
    }
    pub fn simple() -> Self {
        Self::new().examples_systems(true)
    }
    pub fn simple_ui() -> Self {
        Self::new().ui_renderer(true).main_renderer(false).examples_systems(true)
    }
    pub fn simple_dual() -> Self {
        Self::new().ui_renderer(true).main_renderer(true)
    }
    pub fn with_event_loop(mut self, event_loop: EventLoop<()>) -> Self {
        self.event_loop = Some(event_loop);
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

    pub fn examples_systems(mut self, value: bool) -> Self {
        self.examples_systems = value;
        self
    }

    pub async fn build(self) -> anyhow::Result<App> {
        crate::init_all_components();
        let event_loop = self.event_loop.unwrap_or_else(EventLoop::new);
        let window = self.window_builder.unwrap_or_default();
        let window = Arc::new(window.build(&event_loop).unwrap());

        #[cfg(target_os = "unknown")]
        /// Insert a canvas element for the window to attach to
        {
            use winit::platform::web::WindowExtWebSys;

            let canvas = window.canvas();

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let body = document.body().unwrap();

            // Set a background color for the canvas to make it easier to tell where the canvas is for debugging purposes.
            canvas.style().set_css_text("background-color: crimson;");
            body.append_child(&canvas).unwrap();
        }

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

        #[cfg(not(target_os = "unknown"))]
        let _ = thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Max);

        let runtime = RuntimeHandle::current();

        let assets = self.asset_cache.unwrap_or_else(|| AssetCache::new(runtime.clone()));

        let mut world = World::new("main_app");
        let gpu = Arc::new(Gpu::with_config(Some(&window), true).await);

        tracing::info!("Inserting runtime");
        RuntimeKey.insert(&assets, runtime.clone());
        GpuKey.insert(&assets, gpu.clone());
        // WindowKey.insert(&assets, window.clone());

        tracing::info!("Inserting app resources");
        let (ctl_tx, ctl_rx) = flume::unbounded();

        let (window_physical_size, window_logical_size, window_scale_factor) = get_window_sizes(&window);

        let app_resources =
            AppResources { gpu, runtime: runtime.clone(), assets, ctl_tx, window_physical_size, window_logical_size, window_scale_factor };

        let resources = world_instance_resources(app_resources);

        world.add_components(world.resource_entity(), resources).unwrap();
        tracing::info!("Setup renderers");
        if self.ui_renderer || self.main_renderer {
            let _span = info_span!("setup_renderers").entered();
            if !self.main_renderer {
                let renderer = Arc::new(Mutex::new(UIRender::new(&mut world)));
                world.add_resource(ui_renderer(), renderer);
            } else {
                let renderer = Arc::new(Mutex::new(ExamplesRender::new(&mut world, self.ui_renderer, self.main_renderer)));
                world.add_resource(examples_renderer(), renderer);
            }
        }

        let mut window_event_systems = SystemGroup::new(
            "window_event_systems",
            vec![
                Box::new(assets_camera_systems()),
                Box::new(WinitEventsSystem::new()),
                Box::new(ambient_input::event_systems()),
                Box::new(renderers::systems()),
            ],
        );
        if self.examples_systems {
            window_event_systems.add(Box::new(ExamplesSystem));
        }

        Ok(App {
            window_focused: true,
            window,
            runtime,
            systems: SystemGroup::new("app", vec![Box::new(MeshBufferUpdate), Box::new(world_instance_systems(true))]),
            world,
            gpu_world_sync_systems: gpu_world_sync_systems(),
            window_event_systems,
            event_loop: Some(event_loop),

            fps: FpsCounter::new(),
            #[cfg(feature = "profile")]
            _puffin: puffin_server,
            modifiers: Default::default(),
            ctl_rx,
        })
    }

    /// Runs the app by blocking the main thread
    #[cfg(not(target_os = "unknown"))]
    pub fn block_on(self, init: impl for<'x> AsyncInit<'x>) {
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

        rt.block_on(async move {
            let mut app = self.build().await.unwrap();

            init.call(&mut app).await;

            app.run_blocking();
        });
    }

    /// Finalizes the app and enters the main loop
    pub async fn run(self, init: impl FnOnce(&mut App, RuntimeHandle)) {
        let mut app = self.build().await.unwrap();
        let runtime = app.runtime.clone();
        init(&mut app, runtime);
        app.run_blocking()
    }

    #[inline]
    pub async fn run_world(self, init: impl FnOnce(&mut World)) {
        self.run(|app, _| init(&mut app.world)).await
    }
}

pub struct App {
    pub world: World,
    pub ctl_rx: flume::Receiver<WindowCtl>,
    pub systems: SystemGroup,
    pub gpu_world_sync_systems: SystemGroup<GpuWorldSyncEvent>,
    pub window_event_systems: SystemGroup<Event<'static, ()>>,
    pub runtime: RuntimeHandle,
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

    #[cfg(target_os = "unknown")]
    pub fn spawn(mut self) {
        use winit::platform::web::EventLoopExtWebSys;

        let event_loop = self.event_loop.take().unwrap();

        event_loop.spawn(move |event, _, control_flow| {
            tracing::info!("Event: {event:?}");
            // HACK(philpax): treat dpi changes as resize events. Ideally we'd handle this in handle_event proper,
            // but https://github.com/rust-windowing/winit/issues/1968 restricts us
            if let Event::WindowEvent { window_id, event: WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } } = &event {
                *self.world.resource_mut(window_scale_factor()) = *scale_factor;
                self.handle_static_event(
                    &Event::WindowEvent { window_id: *window_id, event: WindowEvent::Resized(**new_inner_size) },
                    control_flow,
                );
            } else if let Some(event) = event.to_static() {
                self.handle_static_event(&event, control_flow);
            }
        });
    }

    pub fn run_blocking(mut self) {
        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow| {
            // HACK(philpax): treat dpi changes as resize events. Ideally we'd handle this in handle_event proper,
            // but https://github.com/rust-windowing/winit/issues/1968 restricts us
            if let Event::WindowEvent { window_id, event: WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } } = &event {
                *self.world.resource_mut(window_scale_factor()) = *scale_factor;
                self.handle_static_event(
                    &Event::WindowEvent { window_id: *window_id, event: WindowEvent::Resized(**new_inner_size) },
                    control_flow,
                );
            } else if let Some(event) = event.to_static() {
                self.handle_static_event(&event, control_flow);
            }
        });
    }

    pub fn handle_static_event(&mut self, event: &Event<'static, ()>, control_flow: &mut ControlFlow) {
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
                // Handle window control events
                for v in self.ctl_rx.try_iter() {
                    tracing::info!("Window control: {v:?}");
                    match v {
                        WindowCtl::GrabCursor(mode) => {
                            self.window.set_cursor_grab(mode).ok();
                        }
                        WindowCtl::ShowCursor(show) => self.window.set_cursor_visible(show),
                        WindowCtl::SetCursorIcon(icon) => self.window.set_cursor_icon(icon),
                    }
                }

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

                    let size = uvec2(size.width, size.height);
                    let logical_size = (size.as_dvec2() * self.window.scale_factor()).as_uvec2();

                    world.set_if_changed(world.resource_entity(), window_physical_size(), size).unwrap();
                    world.set_if_changed(world.resource_entity(), window_logical_size(), logical_size).unwrap();
                }
                WindowEvent::CloseRequested => {
                    tracing::info!("Closing...");
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
    #[allow(clippy::single_match)]
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
