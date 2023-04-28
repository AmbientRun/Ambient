use crate::{
    app::App,
    gpu_world_sync_systems,
    renderers::{self, examples_renderer, ui_renderer, ExamplesRender, UIRender},
    world_instance_resources, world_instance_systems, AppResources, ExamplesSystem,
    MeshBufferUpdate,
};
use ambient_cameras::assets_camera_systems;
use ambient_core::{
    gpu,
    gpu_ecs::GpuWorldSyncEvent,
    window::{
        cursor_position, get_window_sizes, window_logical_size, window_physical_size,
        window_scale_factor, WindowCtl,
    },
    RuntimeKey,
};
use ambient_ecs::{DynSystem, FrameEvent, System, SystemGroup, World};
use ambient_gpu::gpu::{Gpu, GpuKey};
use ambient_input::{ElementState, VirtualKeyCode};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    fps_counter::FpsCounter,
};
use ambient_sys::task::RuntimeHandle;
use glam::{uvec2, vec2, UVec2};
use parking_lot::Mutex;
use std::sync::Arc;
use winit::{
    event::{DeviceId, Event, ModifiersState, WindowEvent},
    event_loop::ControlFlow,
    window::{Fullscreen, Window, WindowId},
};

pub struct AmbientWindowBuilder {
    pub window_builder: Option<winit::window::WindowBuilder>,
    pub asset_cache: Option<AssetCache>,
    pub ui_renderer: bool,
    pub main_renderer: bool,
    pub examples_systems: bool,
    pub size: Option<UVec2>,
    pub update_title_with_fps_stats: bool,
    #[cfg(target_os = "unknown")]
    pub parent_element: Option<web_sys::HtmlElement>,
}

impl AmbientWindowBuilder {
    pub fn new() -> Self {
        Self {
            window_builder: None,
            asset_cache: None,
            ui_renderer: false,
            main_renderer: true,
            examples_systems: false,
            size: None,
            update_title_with_fps_stats: true,
            #[cfg(target_os = "unknown")]
            parent_element: None,
        }
    }

    pub fn with_window_builder(mut self, window_builder: winit::window::WindowBuilder) -> Self {
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

    pub fn size(mut self, value: Option<UVec2>) -> Self {
        self.size = value;
        self
    }

    pub fn update_title_with_fps_stats(mut self, value: bool) -> Self {
        self.update_title_with_fps_stats = value;
        self
    }

    #[cfg(target_os = "unknown")]
    pub fn parent_element(mut self, value: Option<web_sys::HtmlElement>) -> Self {
        self.parent_element = value;
        self
    }

    pub async fn build(self, app: &mut App) -> anyhow::Result<AmbientWindow> {
        crate::init_all_components();
        let window = if let Some(event_loop) = &app.event_loop {
            let window = self.window_builder.unwrap_or_default();
            let window = Arc::new(window.build(event_loop).unwrap());
            Some(window)
        } else {
            None
        };

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

            // Set a background color for the canvas to make it easier to tell where the canvas is for debugging purposes.
            canvas.style().set_css_text("background-color: crimson;");
            target.append_child(&canvas).unwrap();
        }

        let runtime = RuntimeHandle::current();

        let assets = self
            .asset_cache
            .unwrap_or_else(|| AssetCache::new(runtime.clone()));

        let mut world = World::new("main_app");
        let gpu = Arc::new(Gpu::with_config(window.as_deref(), true).await);

        tracing::debug!("Inserting runtime");
        RuntimeKey.insert(&assets, runtime.clone());
        GpuKey.insert(&assets, gpu.clone());
        // WindowKey.insert(&assets, window.clone());

        tracing::debug!("Inserting app resources");
        let (ctl_tx, ctl_rx) = flume::unbounded();

        let (window_physical_size, window_logical_size, window_scale_factor) =
            if let Some(window) = window.as_ref() {
                get_window_sizes(window)
            } else {
                let headless_size = self.size.unwrap();
                (headless_size, headless_size, 1.)
            };

        let app_resources = AppResources {
            gpu,
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
                let renderer = Arc::new(Mutex::new(UIRender::new(&mut world)));
                world.add_resource(ui_renderer(), renderer);
            } else {
                tracing::debug!("Setting up ExamplesRenderer");
                let renderer =
                    ExamplesRender::new(&mut world, self.ui_renderer, self.main_renderer);
                tracing::debug!("Created examples renderer");
                let renderer = Arc::new(Mutex::new(renderer));
                world.add_resource(examples_renderer(), renderer);
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

        Ok(AmbientWindow {
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
            gpu_world_sync_systems: gpu_world_sync_systems(),
            window_event_systems,

            fps: FpsCounter::new(),
            modifiers: Default::default(),
            ctl_rx,
            update_title_with_fps_stats: self.update_title_with_fps_stats,
        })
    }
}

pub struct AmbientWindow {
    pub world: World,
    pub ctl_rx: flume::Receiver<WindowCtl>,
    pub systems: SystemGroup,
    pub gpu_world_sync_systems: SystemGroup<GpuWorldSyncEvent>,
    pub window_event_systems: SystemGroup<winit::event::Event<'static, ()>>,
    pub runtime: RuntimeHandle,
    pub window: Option<Arc<Window>>,
    fps: FpsCounter,
    modifiers: ModifiersState,

    window_focused: bool,
    update_title_with_fps_stats: bool,
}

impl std::fmt::Debug for AmbientWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Window");
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
impl AmbientWindow {
    pub fn builder() -> AmbientWindowBuilder {
        AmbientWindowBuilder::new()
    }
    pub(crate) fn id(&self) -> WindowId {
        if let Some(window) = &self.window {
            window.id()
        } else {
            0.into()
        }
    }

    #[cfg(target_os = "unknown")]
    pub fn spawn(mut self) {
        use winit::platform::web::EventLoopExtWebSys;

        let event_loop = self.event_loop.take().unwrap();

        tracing::debug!("Spawning event loop");
        event_loop.spawn(move |event, _, control_flow| {
            tracing::debug!("Event: {event:?}");
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
                // tracing::info!("Handling event: {event:?}");
                self.handle_static_event(&event, control_flow);
            } else {
                tracing::error!("Failed to convert event to static")
            }
        });
    }

    pub fn add_system(&mut self, system: DynSystem) -> &mut Self {
        self.systems.add(system);
        self
    }

    pub(crate) fn handle_device_event(
        &mut self,
        device_id: DeviceId,
        event: &winit::event::DeviceEvent,
    ) {
        self.window_event_systems.run(
            &mut self.world,
            &winit::event::Event::DeviceEvent {
                device_id,
                event: event.clone(),
            },
        );
    }

    pub(crate) fn handle_event(
        &mut self,
        event: &WindowEvent<'static>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            WindowEvent::Focused(focused) => {
                self.window_focused = *focused;
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                *self.world.resource_mut(window_scale_factor()) = *scale_factor;
            }
            WindowEvent::Resized(size) => {
                let gpu = self.world.resource(gpu()).clone();
                gpu.resize(*size);

                let size = uvec2(size.width, size.height);
                if let Some(window) = &self.window {
                    let scale_factor = window.scale_factor();
                    let logical_size = (size.as_dvec2() / scale_factor).as_uvec2();

                    self.world
                        .set_if_changed(self.world.resource_entity(), window_physical_size(), size)
                        .unwrap();
                    self.world
                        .set_if_changed(
                            self.world.resource_entity(),
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
                    self.world
                        .set(self.world.resource_entity(), cursor_position(), p)
                        .unwrap();
                }
            }
            _ => {}
        }
    }

    pub(crate) fn handle_main_events_cleared(&mut self, control_flow: &mut ControlFlow) {
        // From: https://github.com/gfx-rs/wgpu/issues/1783
        // TODO: According to the issue we should cap the framerate instead
        #[cfg(target_os = "macos")]
        if !self.window_focused {
            *control_flow = ControlFlow::Wait;
        }

        self.world.resource(gpu()).device.poll(wgpu::Maintain::Poll);

        // Handle window control events
        for v in self.ctl_rx.try_iter() {
            tracing::debug!("Window control: {v:?}");
            match v {
                WindowCtl::GrabCursor(mode) => {
                    if let Some(window) = &self.window {
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
            }
        }
        let world = &mut self.world;
        let systems = &mut self.systems;
        let gpu_world_sync_systems = &mut self.gpu_world_sync_systems;
        self.window_event_systems
            .run(world, &Event::MainEventsCleared);

        ambient_profiling::scope!("frame");
        world.next_frame();

        {
            ambient_profiling::scope!("systems");
            systems.run(world, &FrameEvent);
            gpu_world_sync_systems.run(world, &GpuWorldSyncEvent);
        }

        if let Some(fps) = self.fps.frame_next() {
            world
                .set(world.resource_entity(), crate::fps_stats(), fps.clone())
                .unwrap();
            if self.update_title_with_fps_stats {
                if let Some(window) = &self.window {
                    window.set_title(&format!(
                        "{} [{}, {} entities]",
                        world.resource(crate::window_title()),
                        fps.dump_both(),
                        world.len()
                    ));
                }
            }
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
        ambient_profiling::finish_frame!();
    }
}
