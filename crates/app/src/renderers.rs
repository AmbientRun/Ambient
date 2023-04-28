use std::sync::Arc;

use ambient_core::{asset_cache, gpu, main_scene, ui_scene, window::window_physical_size};
use ambient_ecs::{components, query, FrameEvent, System, SystemGroup, World};
use ambient_gizmos::render::GizmoRenderer;
use ambient_gpu::{
    blit::{Blitter, BlitterKey},
    gpu::Gpu,
    shader_module::DEPTH_FORMAT,
    texture::{Texture, TextureView},
};
use ambient_renderer::{renderer_stats, RenderTarget, Renderer, RendererConfig, RendererTarget};
use ambient_std::{asset_cache::SyncAssetKeyExt, color::Color};
use ambient_ui_native::app_background_color;
use glam::{uvec2, UVec2};
use parking_lot::Mutex;
use wgpu::FilterMode;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
};

components!("app_renderers", {
    ui_renderer: Arc<Mutex<UIRender>>,
    examples_renderer: Arc<Mutex<ExamplesRender>>,
});

pub fn systems() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new(
        "app_renderers",
        vec![
            query(ui_renderer()).to_system(|q, world, qs, event| {
                for (_, ui_render) in q.collect_cloned(world, qs) {
                    let mut ui_render = ui_render.lock();
                    match &event {
                        Event::WindowEvent {
                            event: WindowEvent::Resized(size),
                            ..
                        } => ui_render.resize(size),
                        Event::WindowEvent {
                            event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                            ..
                        } => {
                            ui_render.resize(new_inner_size);
                        }
                        _ => {}
                    }
                    let cleared = matches!(event, Event::MainEventsCleared);
                    if cleared {
                        ui_render.render(world);
                    }
                }
            }),
            query(examples_renderer()).to_system(|q, world, qs, event| {
                for (_, examples_render) in q.collect_cloned(world, qs) {
                    let mut examples_render = examples_render.lock();
                    match event {
                        Event::WindowEvent {
                            event: WindowEvent::Resized(size),
                            ..
                        } => examples_render.resize(size),
                        Event::MainEventsCleared => {
                            examples_render.run(world, &FrameEvent);
                        }
                        _ => {}
                    }
                }
            }),
        ],
    )
}

pub struct ExamplesRender {
    gpu: Arc<Gpu>,
    main: Option<Renderer>,
    ui: Option<Renderer>,
    blit: Arc<Blitter>,
    render_target: RenderTarget,
    size: UVec2,
}

impl ExamplesRender {
    pub fn new(world: &mut World, ui: bool, main: bool) -> Self {
        let gpu = world.resource(gpu()).clone();
        let assets = world.resource(asset_cache()).clone();
        world
            .add_component(world.resource_entity(), renderer_stats(), "".to_string())
            .unwrap();
        let wind_size = *world.resource(ambient_core::window::window_physical_size());

        tracing::debug!("Creating render target");
        let render_target = RenderTarget::new(gpu.clone(), wind_size, None);

        tracing::debug!("Creating self");

        let is_srgb = gpu.swapchain_format().describe().srgb;
        let gamma_correction = if !is_srgb {
            tracing::info!(
                "Output format is not in srgb colorspace. Applying manual gamma correction"
            );
            Some(2.2)
        } else {
            None
        };

        Self {
            main: if main {
                tracing::debug!("Creating renderer");
                let mut renderer = Renderer::new(
                    world,
                    world.resource(asset_cache()).clone(),
                    RendererConfig {
                        scene: main_scene(),
                        shadows: true,
                        ..Default::default()
                    },
                );

                tracing::debug!("Creating gizmo renderer");
                renderer.post_transparent = Some(Box::new(GizmoRenderer::new(&assets)));
                Some(renderer)
            } else {
                None
            },
            ui: if ui {
                Some(Renderer::new(
                    world,
                    world.resource(asset_cache()).clone(),
                    RendererConfig {
                        scene: ui_scene(),
                        shadows: false,
                        ..Default::default()
                    },
                ))
            } else {
                None
            },
            blit: BlitterKey {
                format: gpu.swapchain_format().into(),
                min_filter: FilterMode::Nearest,
                gamma_correction,
            }
            .get(&world.resource(asset_cache()).clone()),
            render_target,
            gpu,
            size: wind_size,
        }
    }
    fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.size = uvec2(size.width, size.height);

        if size.width > 0 && size.height > 0 {
            self.render_target =
                RenderTarget::new(self.gpu.clone(), uvec2(size.width, size.height), None);
        }
    }

    pub fn dump_to_tmp_file(&self) {
        std::fs::create_dir_all("tmp").unwrap();
        let mut f = std::fs::File::create("tmp/renderer.txt").expect("Unable to create file");
        self.dump(&mut f);
        tracing::info!("Wrote renderer to tmp/renderer.txt");
    }
    #[allow(dead_code)]
    pub fn n_entities(&self) -> usize {
        self.main.as_ref().map(|x| x.n_entities()).unwrap_or(0)
            + self.ui.as_ref().map(|x| x.n_entities()).unwrap_or(0)
    }
    pub fn stats(&self) -> String {
        if let Some(main) = &self.main {
            main.stats()
        } else {
            String::new()
        }
    }
    pub fn dump(&self, f: &mut dyn std::io::Write) {
        if let Some(main) = &self.main {
            writeln!(f, "## MAIN ##").unwrap();
            main.dump(f);
        }
        if let Some(ui) = &self.ui {
            writeln!(f, "## UI ##").unwrap();
            ui.dump(f);
        }
    }
}

impl std::fmt::Debug for ExamplesRender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer").finish()
    }
}

impl System for ExamplesRender {
    fn run(&mut self, world: &mut World, _: &FrameEvent) {
        // tracing::info!("ExamplesRenderer");
        ambient_profiling::scope!("Renderers.run");
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let mut post_submit = Vec::new();

        if let Some(main) = &mut self.main {
            ambient_profiling::scope!("Main");
            main.render(
                world,
                &mut encoder,
                &mut post_submit,
                RendererTarget::Target(&self.render_target),
                Some(Color::rgba(0.0, 0., 0.0, 1.)),
            );
        }

        if let Some(ui) = &mut self.ui {
            // tracing::info!("Drawing UI");
            ambient_profiling::scope!("UI");
            ui.render(
                world,
                &mut encoder,
                &mut post_submit,
                RendererTarget::Target(&self.render_target),
                if self.main.is_some() {
                    None
                } else {
                    Some(app_background_color())
                },
            );
        }

        if let Some(surface) = &self.gpu.surface {
            if self.size.x > 0 && self.size.y > 0 {
                let frame = {
                    ambient_profiling::scope!("Get swapchain texture");
                    match surface.get_current_texture() {
                        Ok(v) => v,
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => {
                            tracing::warn!("Surface lost");
                            self.gpu.resize(PhysicalSize {
                                width: self.size.x,
                                height: self.size.y,
                            });
                            return;
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of memory"),
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(err) => {
                            tracing::warn!("{err:?}");
                            return;
                        }
                    }
                };
                let frame_view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.blit.run(
                    &mut encoder,
                    &self.render_target.color_buffer_view,
                    &frame_view,
                );

                {
                    ambient_profiling::scope!("Submit");
                    self.gpu.queue.submit(Some(encoder.finish()));
                }
                {
                    ambient_profiling::scope!("Present");
                    frame.present();
                }
            } else {
                ambient_profiling::scope!("Submit");
                self.gpu.queue.submit(Some(encoder.finish()));
            }
        } else {
            {
                ambient_profiling::scope!("Submit");
                self.gpu.queue.submit(Some(encoder.finish()));
            }
        }

        for action in post_submit.into_iter() {
            action();
        }

        world
            .set(world.resource_entity(), renderer_stats(), self.stats())
            .unwrap();
    }
}

pub struct UIRender {
    gpu: Arc<Gpu>,
    ui_renderer: Renderer,
    depth_buffer_view: Arc<TextureView>,
    normals_view: Arc<TextureView>,
}

impl UIRender {
    pub fn new(world: &mut World) -> Self {
        let gpu = world.resource(gpu()).clone();
        let size = *world.resource(window_physical_size());

        let depth_buffer = Arc::new(Self::create_depth_buffer(
            gpu.clone(),
            &PhysicalSize::new(size.x, size.y),
        ));

        let normals = Arc::new(Texture::new(
            gpu.clone(),
            &wgpu::TextureDescriptor {
                label: Some("RenderTarget.depth_buffer"),
                size: wgpu::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Snorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            },
        ));

        let assets = world.resource(asset_cache()).clone();
        let mut ui_renderer = Renderer::new(
            world,
            world.resource(asset_cache()).clone(),
            RendererConfig {
                scene: ui_scene(),
                shadows: false,
                ..Default::default()
            },
        );
        ui_renderer.post_transparent = Some(Box::new(GizmoRenderer::new(&assets)));
        Self {
            ui_renderer,
            depth_buffer_view: Arc::new(depth_buffer.create_view(&Default::default())),
            gpu,
            normals_view: Arc::new(normals.create_view(&Default::default())),
        }
    }

    fn create_depth_buffer(gpu: Arc<Gpu>, size: &PhysicalSize<u32>) -> Texture {
        Texture::new(
            gpu,
            &wgpu::TextureDescriptor {
                label: Some("RenderTarget.depth_buffer"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            },
        )
    }

    fn resize(&mut self, size: &PhysicalSize<u32>) {
        let depth_buffer = Arc::new(Self::create_depth_buffer(self.gpu.clone(), size));
        self.depth_buffer_view = Arc::new(depth_buffer.create_view(&Default::default()));
    }

    fn render(&mut self, world: &mut World) {
        let gpu = world.resource(gpu()).clone();
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UIRenderer"),
            });
        let frame = {
            ambient_profiling::scope!("Get swapchain texture");
            gpu.surface
                .as_ref()
                .unwrap()
                .get_current_texture()
                .expect("Failed to acquire next swap chain texture")
        };

        let window_size = world.resource(window_physical_size());
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut post_submit = Vec::new();

        tracing::info!("Drawing UI");

        self.ui_renderer.render(
            world,
            &mut encoder,
            &mut post_submit,
            RendererTarget::Direct {
                color: &frame_view,
                depth: &self.depth_buffer_view,
                size: wgpu::Extent3d {
                    width: window_size.x,
                    height: window_size.y,
                    depth_or_array_layers: 1,
                },
                normals: &self.normals_view,
            },
            Some(app_background_color()),
        );
        {
            ambient_profiling::scope!("Submit");
            gpu.queue.submit(Some(encoder.finish()));
        }
        {
            ambient_profiling::scope!("Present");
            frame.present();
        }
        for action in post_submit {
            action();
        }
    }
}
