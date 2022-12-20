use std::sync::Arc;

use elements_core::{asset_cache, gpu, main_scene, ui_scene};
use elements_ecs::{FrameEvent, System, World};
use elements_gizmos::render::GizmoRenderer;
use elements_gpu::{
    blit::{Blitter, BlitterKey}, gpu::Gpu
};
use elements_renderer::{get_screen_render_target, renderer_stats, Renderer, RendererConfig, RendererTarget};
use elements_std::{asset_cache::SyncAssetKeyExt, color::Color};
use elements_ui::app_background_color;

pub struct ExamplesRender {
    gpu: Arc<Gpu>,
    main: Option<Renderer>,
    ui: Option<Renderer>,
    blit: Arc<Blitter>,
}

impl ExamplesRender {
    pub fn new(world: &mut World, ui: bool, main: bool) -> Self {
        let gpu = world.resource(gpu()).clone();
        let assets = world.resource(asset_cache()).clone();
        world.add_component(world.resource_entity(), renderer_stats(), "".to_string()).unwrap();

        Self {
            main: if main {
                Some(Renderer::new(
                    world,
                    world.resource(asset_cache()).clone(),
                    RendererConfig {
                        scene: main_scene(),
                        shadows: true,
                        post_transparent: Some(Box::new(GizmoRenderer::new(&assets))),
                        ..Default::default()
                    },
                ))
            } else {
                None
            },
            ui: if ui {
                Some(Renderer::new(
                    world,
                    world.resource(asset_cache()).clone(),
                    RendererConfig { scene: ui_scene(), shadows: false, ..Default::default() },
                ))
            } else {
                None
            },
            blit: BlitterKey { format: gpu.swapchain_format().into(), linear: false }.get(&world.resource(asset_cache()).clone()),
            gpu,
        }
    }

    pub fn dump_to_tmp_file(&self) {
        std::fs::create_dir_all("tmp").unwrap();
        let mut f = std::fs::File::create("tmp/renderer.txt").expect("Unable to create file");
        self.dump(&mut f);
        println!("Wrote renderer to tmp/renderer.txt");
    }
    pub fn n_entities(&self) -> usize {
        self.main.as_ref().map(|x| x.n_entities()).unwrap_or(0) + self.ui.as_ref().map(|x| x.n_entities()).unwrap_or(0)
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
        profiling::scope!("Renderers.run");
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let mut post_submit = Vec::new();

        let screen = get_screen_render_target(world.resource(asset_cache()).clone());
        if let Some(main) = &mut self.main {
            profiling::scope!("Main");
            main.render(world, &mut encoder, &mut post_submit, RendererTarget::Target(&screen), Some(Color::rgba(0., 0., 0., 1.)));
        }
        if let Some(ui) = &mut self.ui {
            profiling::scope!("UI");
            ui.render(
                world,
                &mut encoder,
                &mut post_submit,
                RendererTarget::Target(&screen),
                if self.main.is_some() { None } else { Some(app_background_color()) },
            );
        }
        let frame = {
            profiling::scope!("Get swapchain texture");
            self.gpu.surface.as_ref().unwrap().get_current_texture().expect("Failed to acquire next swap chain texture")
        };
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.blit.run(&mut encoder, &screen.screen_buffer_view, &frame_view);

        {
            profiling::scope!("Submit");
            self.gpu.queue.submit(Some(encoder.finish()));
        }
        {
            profiling::scope!("Present");
            frame.present();
        }
        for action in post_submit.into_iter() {
            action();
        }
        world.set(world.resource_entity(), renderer_stats(), self.stats()).unwrap();
    }
}
