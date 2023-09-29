use std::{future::Future, sync::Arc, time::Duration};

use ambient_cameras::assets_camera_systems;
pub use ambient_core::gpu;
use ambient_core::{
    asset_cache,
    async_ecs::async_ecs_systems,
    bounding::bounding_systems,
    camera::camera_systems,
    frame_index,
    hierarchy::dump_world_hierarchy_to_user,
    name, performance_samples, refcount_system, remove_at_time_system, runtime,
    transform::TransformSystem,
    window::{
        cursor_position, get_window_sizes, window_logical_size, window_physical_size,
        window_scale_factor, ExitStatus, WindowCtl,
    },
    ClientTimeResourcesSystem, PerformanceSample, RuntimeKey,
};
use ambient_ecs::{
    components, generated::ui::components::focus, world_events, Debuggable, DynSystem, Entity,
    FrameEvent, MakeDefault, MaybeResource, System, SystemGroup, World, WorldEventsSystem,
};
use ambient_element::ambient_system;
use ambient_gizmos::{gizmos, Gizmos};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    mesh_buffer::MeshBufferKey,
};
use ambient_gpu_ecs::{gpu_world, GpuWorld, GpuWorldSyncEvent, GpuWorldUpdate};
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    fps_counter::{FpsCounter, FpsSample},
};
use ambient_procedurals::{procedural_storage, ProceduralStorage};
use ambient_renderer::lod::lod_system;
use ambient_settings::SettingsKey;
use ambient_sys::{task::RuntimeHandle, time::Instant};

use glam::{uvec2, vec2, IVec2, UVec2, Vec2};
use parking_lot::Mutex;
use renderers::{main_renderer, ui_renderer, MainRenderer, UiRenderer};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, Fullscreen, Window, WindowBuilder},
};

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
    ambient_animation::init_all_components();
    ambient_gizmos::init_components();
    ambient_cameras::init_all_components();
    init_components();
    ambient_renderer::init_all_components();
    ambient_ui_native::init_all_components();
    ambient_input::init_all_components();
    ambient_model::init_components();
    ambient_cameras::init_all_components();
    renderers::init_components();
    ambient_procedurals::init_components();
}

pub fn gpu_world_sync_systems(gpu: Arc<Gpu>) -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "gpu_world",
        vec![
            // Note: All Gpu sync systems must run immediately after GpuWorldUpdate, as that's the only time we know
            // the layout of the GpuWorld is correct
            Box::new(GpuWorldUpdate(gpu.clone())),
            Box::new(ambient_core::transform::transform_gpu_systems(gpu.clone())),
            Box::new(ambient_renderer::gpu_world_systems(gpu.clone())),
            Box::new(ambient_core::bounding::gpu_world_systems(gpu.clone())),
            Box::new(ambient_ui_native::layout::gpu_world_systems(gpu.clone())),
        ],
    )
}

pub fn world_instance_systems(full: bool) -> SystemGroup {
    SystemGroup::new(
        "world_instance",
        vec![
            Box::new(ClientTimeResourcesSystem::new()),
            Box::new(async_ecs_systems()),
            remove_at_time_system(),
            refcount_system(),
            Box::new(ambient_core::hierarchy::systems()),
            Box::new(WorldEventsSystem),
            Box::new(ambient_focus::systems()),
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
            Box::new(ambient_procedurals::client_systems()),
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
    Entity::new()
        .with(name(), "Resources".to_string())
        .with(self::gpu(), resources.gpu.clone())
        .with(gizmos(), Gizmos::new())
        .with(self::runtime(), resources.runtime)
        .with(self::window_title(), "".to_string())
        .with(self::fps_stats(), FpsSample::default())
        .with(self::performance_samples(), Vec::new())
        .with(self::asset_cache(), resources.assets.clone())
        .with(world_events(), Default::default())
        .with(frame_index(), 0_usize)
        .with(ambient_core::window::cursor_position(), Vec2::ZERO)
        .with(
            gpu_world(),
            GpuWorld::new_arced(&resources.gpu, resources.assets),
        )
        .with_merge(ambient_core::time_resources_start(Duration::ZERO))
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
        .with(procedural_storage(), ProceduralStorage::new())
        .with(focus(), Default::default())
}

pub struct AppBuilder {
    pub event_loop: Option<EventLoop<()>>,
    pub asset_cache: Option<AssetCache>,
    pub ui_renderer: bool,
    pub main_renderer: bool,
    pub examples_systems: bool,
    pub headless: Option<UVec2>,
    pub update_title_with_fps_stats: bool,
    ctl: Option<(flume::Sender<WindowCtl>, flume::Receiver<WindowCtl>)>,
    pub window_position_override: Option<IVec2>,
    pub window_size_override: Option<UVec2>,
    #[cfg(target_os = "unknown")]
    pub parent_element: Option<web_sys::HtmlElement>,
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
        Self {
            event_loop: None,
            asset_cache: None,
            ui_renderer: false,
            main_renderer: true,
            examples_systems: false,
            headless: None,
            update_title_with_fps_stats: true,
            ctl: None,
            window_position_override: None,
            window_size_override: None,
            #[cfg(target_os = "unknown")]
            parent_element: None,
        }
    }
    pub fn simple() -> Self {
        Self::new().examples_systems(true)
    }
    pub fn simple_ui() -> Self {
        Self::new()
            .ui_renderer(true)
            .main_renderer(false)
            .examples_systems(true)
    }
    pub fn simple_dual() -> Self {
        Self::new().ui_renderer(true).main_renderer(true)
    }
    pub fn with_event_loop(mut self, event_loop: EventLoop<()>) -> Self {
        self.event_loop = Some(event_loop);
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

    pub fn headless(mut self, value: Option<UVec2>) -> Self {
        self.headless = value;
        self
    }

    pub fn update_title_with_fps_stats(mut self, value: bool) -> Self {
        self.update_title_with_fps_stats = value;
        self
    }

    pub fn with_window_position_override(mut self, position: IVec2) -> Self {
        self.window_position_override = Some(position);
        self
    }

    pub fn with_window_size_override(mut self, size: UVec2) -> Self {
        self.window_size_override = Some(size);
        self
    }

    #[cfg(target_os = "unknown")]
    pub fn parent_element(mut self, value: Option<web_sys::HtmlElement>) -> Self {
        self.parent_element = value;
        self
    }

    pub fn window_ctl(
        mut self,
        ctl_tx: flume::Sender<WindowCtl>,
        ctl_rx: flume::Receiver<WindowCtl>,
    ) -> Self {
        self.ctl = Some((ctl_tx, ctl_rx));

        self
    }

    pub async fn build(self) -> anyhow::Result<App> {
        crate::init_all_components();

        let runtime = RuntimeHandle::current();

        let assets = self
            .asset_cache
            .unwrap_or_else(|| AssetCache::new(runtime.clone()));

        let settings = SettingsKey.get(&assets);

        let (window, event_loop) = if self.headless.is_some() {
            (None, None)
        } else {
            let event_loop = self.event_loop.unwrap_or_else(EventLoop::new);
            let window = WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize {
                width: settings.render.resolution().0,
                height: settings.render.resolution().1,
            });
            let window = if let Some(position) = self.window_position_override {
                window.with_position(winit::dpi::LogicalPosition {
                    x: position.x,
                    y: position.y,
                })
            } else {
                window
            };
            let window = if let Some(size) = self.window_size_override {
                window.with_inner_size(winit::dpi::LogicalSize {
                    width: size.x,
                    height: size.y,
                })
            } else {
                window
            };
            let window = Arc::new(window.build(&event_loop).unwrap());
            (Some(window), Some(event_loop))
        };

        #[cfg(target_os = "unknown")]
        let mut drop_handles: Vec<Box<dyn std::fmt::Debug>> = Vec::new();

        #[cfg(target_os = "unknown")]
        // Insert a canvas element for the window to attach to
        if let Some(window) = &window {
            use winit::platform::web::WindowExtWebSys;

            let canvas = window.canvas();

            let target = self.parent_element.unwrap_or_else(|| {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                document.body().unwrap()
            });

            use wasm_bindgen::prelude::*;

            let on_context_menu = Closure::<dyn Fn(_)>::new(|event: web_sys::MouseEvent| {
                event.prevent_default();
            });

            canvas.set_oncontextmenu(Some(on_context_menu.as_ref().unchecked_ref()));

            drop_handles.push(Box::new(on_context_menu));

            // Get the screen's available width and height
            let window = web_sys::window().unwrap();

            let max_width = target.client_width();
            let max_height = target.client_height();

            // Get device pixel ratio
            let device_pixel_ratio = window.device_pixel_ratio();

            // Calculate the real dimensions of the canvas considering the device pixel ratio
            let real_width = (max_width as f64 * device_pixel_ratio) as u32;
            let real_height = (max_height as f64 * device_pixel_ratio) as u32;

            // Set the canvas dimensions using the real dimensions
            canvas.set_width(real_width);
            canvas.set_height(real_height);

            // Set a background color for the canvas to make it easier to tell where the canvas is for debugging purposes.
            // Use the maximum available width and height as the canvas dimensions.
            canvas.style().set_css_text(&format!(
                "background-color: black; width: {}px; height: {}px; z-index: 50",
                max_width, max_height
            ));

            target.append_child(&canvas).unwrap();
        }

        #[cfg(feature = "profile")]
        let puffin_server = {
            let puffin_addr = format!(
                "0.0.0.0:{}",
                std::env::var("PUFFIN_PORT")
                    .ok()
                    .and_then(|port| port.parse::<u16>().ok())
                    .unwrap_or(puffin_http::DEFAULT_PORT)
            );
            match puffin_http::Server::new(&puffin_addr) {
                Ok(server) => {
                    tracing::debug!("Puffin server running on {}", puffin_addr);
                    puffin::set_scopes_on(true);
                    Some(server)
                }
                Err(err) => {
                    tracing::error!("Failed to start puffin server: {:?}", err);
                    None
                }
            }
        };

        #[cfg(not(target_os = "unknown"))]
        let _ = thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Max);

        let mut world = World::new("main_app", ambient_ecs::WorldContext::App);
        let gpu = Arc::new(Gpu::with_config(window.as_deref(), true, &settings.render).await);

        tracing::debug!("Inserting runtime");
        RuntimeKey.insert(&assets, runtime.clone());
        GpuKey.insert(&assets, gpu.clone());
        // WindowKey.insert(&assets, window.clone());

        tracing::debug!("Inserting app resources");
        let (ctl_tx, ctl_rx) = self.ctl.unwrap_or_else(flume::unbounded);

        let (window_physical_size, window_logical_size, window_scale_factor) =
            if let Some(window) = window.as_ref() {
                get_window_sizes(window)
            } else {
                let headless_size = self.headless.unwrap();
                (headless_size, headless_size, 1.)
            };

        let app_resources = AppResources {
            gpu: gpu.clone(),
            runtime: runtime.clone(),
            assets,
            ctl_tx,
            window_physical_size,
            window_logical_size,
            window_scale_factor,
        };

        let resources = world_instance_resources(app_resources);

        world
            .add_components(world.resource_entity(), resources)
            .unwrap();
        tracing::debug!("Setup renderers");
        if self.ui_renderer || self.main_renderer {
            // let _span = info_span!("setup_renderers").entered();
            if !self.main_renderer {
                tracing::debug!("Setting up UI renderer");
                let renderer = Arc::new(Mutex::new(UiRenderer::new(&mut world)));
                world.add_resource(ui_renderer(), renderer);
            } else {
                tracing::debug!("Setting up Main renderer");
                let renderer =
                    MainRenderer::new(&gpu, &mut world, self.ui_renderer, self.main_renderer);
                tracing::debug!("Created main renderer");
                let renderer = Arc::new(Mutex::new(renderer));
                world.add_resource(main_renderer(), renderer);
            }
        }

        tracing::debug!("Adding window event systems");

        let mut window_event_systems = SystemGroup::new(
            "window_event_systems",
            vec![
                Box::new(assets_camera_systems()),
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
            systems: SystemGroup::new(
                "app",
                vec![
                    Box::new(MeshBufferUpdate),
                    Box::new(world_instance_systems(true)),
                ],
            ),
            world,
            gpu_world_sync_systems: gpu_world_sync_systems(gpu.clone()),
            window_event_systems,
            event_loop,

            fps: FpsCounter::new(),
            #[cfg(feature = "profile")]
            _puffin: puffin_server,
            modifiers: Default::default(),
            ctl_rx,
            current_time: Instant::now(),
            update_title_with_fps_stats: self.update_title_with_fps_stats,
            #[cfg(target_os = "unknown")]
            _drop_handles: drop_handles,
        })
    }

    /// Runs the app by blocking the main thread
    #[cfg(not(target_os = "unknown"))]
    pub fn block_on(self, init: impl for<'x> AsyncInit<'x>) {
        let rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();

        rt.block_on(async move {
            let mut app = self.build().await.unwrap();

            init.call(&mut app).await;

            app.run_blocking();
        });
    }

    /// Finalizes the app and enters the main loop
    pub async fn run(self, init: impl FnOnce(&mut App, RuntimeHandle)) -> ExitStatus {
        let mut app = self.build().await.unwrap();
        let runtime = app.runtime.clone();
        init(&mut app, runtime);
        app.run_blocking()
    }

    #[inline]
    pub async fn run_world(self, init: impl FnOnce(&mut World)) -> ExitStatus {
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
    pub window: Option<Arc<Window>>,
    event_loop: Option<EventLoop<()>>,
    fps: FpsCounter,
    #[cfg(feature = "profile")]
    _puffin: Option<puffin_http::Server>,
    modifiers: ModifiersState,

    window_focused: bool,
    update_title_with_fps_stats: bool,
    #[cfg(target_os = "unknown")]
    _drop_handles: Vec<Box<dyn std::fmt::Debug>>,
    current_time: Instant,
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

        tracing::debug!("Spawning event loop");
        event_loop.spawn(move |event, _, control_flow| {
            // HACK(philpax): treat dpi changes as resize events. Ideally we'd handle this in handle_event proper,
            // but https://github.com/rust-windowing/winit/issues/1968 restricts us
            if let Event::WindowEvent {
                window_id,
                event:
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        scale_factor,
                    },
            } = &event
            {
                *self.world.resource_mut(window_scale_factor()) = *scale_factor;
                self.handle_static_event(
                    &Event::WindowEvent {
                        window_id: *window_id,
                        event: WindowEvent::Resized(**new_inner_size),
                    },
                    control_flow,
                );
            } else if let Some(event) = event.to_static() {
                self.handle_static_event(&event, control_flow);
            } else {
                tracing::error!("Failed to convert event to static")
            }
        });
    }

    pub fn run_blocking(mut self) -> ExitStatus {
        if let Some(event_loop) = self.event_loop.take() {
            event_loop.run(move |event, _, control_flow| {
                // HACK(philpax): treat dpi changes as resize events. Ideally we'd handle this in handle_event proper,
                // but https://github.com/rust-windowing/winit/issues/1968 restricts us
                if let Event::WindowEvent {
                    window_id,
                    event:
                        WindowEvent::ScaleFactorChanged {
                            new_inner_size,
                            scale_factor,
                        },
                } = &event
                {
                    *self.world.resource_mut(window_scale_factor()) = *scale_factor;
                    self.handle_static_event(
                        &Event::WindowEvent {
                            window_id: *window_id,
                            event: WindowEvent::Resized(**new_inner_size),
                        },
                        control_flow,
                    );
                } else if let Some(event) = event.to_static() {
                    self.handle_static_event(&event, control_flow);
                }
            });
        } else {
            // Fake event loop in headless mode
            loop {
                let mut control_flow = ControlFlow::default();
                let exit_status =
                    self.handle_static_event(&Event::MainEventsCleared, &mut control_flow);
                if control_flow == ControlFlow::Exit {
                    return exit_status;
                }
            }
        }
    }

    pub fn handle_static_event(
        &mut self,
        event: &Event<'static, ()>,
        control_flow: &mut ControlFlow,
    ) -> ExitStatus {
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
                let frame_start = Instant::now();
                let external_time = frame_start.duration_since(self.current_time);

                tracing::trace!(?event, "event");

                // Handle window control events
                for v in self.ctl_rx.try_iter() {
                    tracing::trace!(?v, "window control");
                    match v {
                        WindowCtl::GrabCursor(mode) => {
                            if let Some(window) = &self.window {
                                match mode {
                                    CursorGrabMode::Confined | CursorGrabMode::Locked => {
                                        // Move the cursor to the centre of the window to ensure
                                        // the cursor is within the window and will not be locked
                                        // in place outside the window.
                                        //
                                        // Without this, on macOS, the cursor will be locked in place
                                        // and visible outside the window, which means the user can
                                        // click on other aspects of the operating system while
                                        // the cursor is locked.
                                        let (width, height) =
                                            <(u32, u32)>::from(window.inner_size());
                                        window
                                            .set_cursor_position(PhysicalPosition::new(
                                                width / 2,
                                                height / 2,
                                            ))
                                            .ok();
                                    }
                                    _ => {}
                                }
                                window.set_cursor_grab(mode).ok();
                            }
                        }
                        WindowCtl::ShowCursor(show) => {
                            if let Some(window) = &self.window {
                                window.set_cursor_visible(show);
                            }
                        }
                        WindowCtl::SetCursorIcon(icon) => {
                            if let Some(window) = &self.window {
                                window.set_cursor_icon(icon);
                            }
                        }
                        WindowCtl::SetTitle(title) => {
                            if let Some(window) = &self.window {
                                window.set_title(&title);
                            }
                        }
                        WindowCtl::SetFullscreen(fullscreen) => {
                            if let Some(window) = &self.window {
                                window.set_fullscreen(if fullscreen {
                                    Some(Fullscreen::Borderless(None))
                                } else {
                                    None
                                });
                            }
                        }
                        WindowCtl::ExitProcess(exit_status) => {
                            *control_flow = ControlFlow::Exit;
                            return exit_status;
                        }
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
                    world
                        .set(world.resource_entity(), self::fps_stats(), fps.clone())
                        .unwrap();
                    if self.update_title_with_fps_stats {
                        if let Some(window) = &self.window {
                            window.set_title(&format!(
                                "{} [{}, {} entities]",
                                world.resource(window_title()),
                                fps.dump_both(),
                                world.len()
                            ));
                        }
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }

                let frame_end = Instant::now();

                let frame_time = frame_end.duration_since(self.current_time);

                tracing::debug!(?external_time, ?frame_time, "frame time");
                self.current_time = frame_end;

                let samples = world.resource_mut(performance_samples());

                if samples.len() >= 128 {
                    samples.remove(0);
                }

                samples.push(PerformanceSample {
                    frame_time,
                    external_time,
                });

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
                    if let Some(window) = &self.window {
                        let scale_factor = window.scale_factor();
                        let logical_size = (size.as_dvec2() / scale_factor).as_uvec2();

                        world
                            .set_if_changed(world.resource_entity(), window_physical_size(), size)
                            .unwrap();
                        world
                            .set_if_changed(
                                world.resource_entity(),
                                window_logical_size(),
                                logical_size,
                            )
                            .unwrap();
                    }
                }
                WindowEvent::CloseRequested => {
                    tracing::debug!("Closing...");
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
                        let p = vec2(position.x as f32, position.y as f32)
                            / self
                                .window
                                .as_ref()
                                .map(|x| x.scale_factor() as f32)
                                .unwrap_or(1.);
                        world
                            .set(world.resource_entity(), cursor_position(), p)
                            .unwrap();
                    }
                }
                _ => {}
            },
            _ => {}
        }
        ExitStatus::SUCCESS
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
                VirtualKeyCode::F1 => dump_world_hierarchy_to_user(world),
                #[cfg(not(target_os = "unknown"))]
                VirtualKeyCode::F2 => world.dump_to_tmp_file(),
                #[cfg(not(target_os = "unknown"))]
                VirtualKeyCode::F3 => world.resource(main_renderer()).lock().dump_to_tmp_file(),
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
        let gpu = world.resource(gpu()).clone();
        let mesh_buffer = MeshBufferKey.get(&assets);
        let mut mesh_buffer = mesh_buffer.lock();
        mesh_buffer.update(&gpu);
    }
}

#[derive(Debug)]
pub struct DummySystem;
impl System for DummySystem {
    fn run(&mut self, _world: &mut World, _event: &FrameEvent) {}
}
