use std::sync::Arc;

use ambient_app::{gpu_world_sync_systems, world_instance_systems};
use ambient_core::{
    camera::{get_active_camera, projection_view},
    gpu_ecs::GpuWorldSyncEvent,
    main_scene,
    transform::local_to_world,
    ui_scene,
    window::window_physical_size,
};
use ambient_ecs::{components, query, Entity, FrameEvent, System, SystemGroup, World};
use ambient_gizmos::render::GizmoRenderer;
use ambient_gpu::gpu::Gpu;
use ambient_native_std::{asset_cache::AssetCache, color::Color, math::interpolate, shapes::Ray};
use ambient_renderer::{RenderTarget, Renderer, RendererConfig, RendererTarget};
use ambient_world_audio::systems::{setup_audio, spatial_audio_systems};
use glam::{vec2, Mat4, Vec2, Vec3, Vec3Swizzles};

use ambient_core::player::{player, user_id};
use tracing::debug_span;

components!("rendering", {
    game_screen_render_target: Arc<RenderTarget>,
});

#[derive(Debug)]
/// Holds the material world of the client.
pub struct ClientGameState {
    pub world: World,
    systems: SystemGroup,
    temporary_systems: Vec<TempSystem>,
    gpu_world_sync_systems: SystemGroup<GpuWorldSyncEvent>,
    pub renderer: Renderer,
    pub ui_renderer: Renderer,
    pub(crate) assets: AssetCache,
    user_id: String,
}

struct TempSystem(Box<dyn FnMut(&mut World) -> bool + Sync + Send>);

impl std::fmt::Debug for TempSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TempSystem").finish()
    }
}

impl ClientGameState {
    pub fn new(
        gpu: &Gpu,
        assets: AssetCache,
        player_id: String,
        client_systems: SystemGroup,
        all_resources: Entity,
    ) -> Self {
        let mut game_world = World::new("client_game_world");
        setup_audio(&mut game_world).unwrap();

        game_world
            .add_components(game_world.resource_entity(), all_resources)
            .unwrap();

        let systems = SystemGroup::new(
            "game",
            vec![
                Box::new(client_systems),
                Box::new(world_instance_systems(true)),
                Box::new(spatial_audio_systems()),
                Box::new(ambient_world_audio::systems::audio_systems()),
            ],
        );
        let mut renderer = Renderer::new(
            gpu,
            &assets,
            RendererConfig {
                scene: main_scene(),
                shadows: true,
                ..Default::default()
            },
        );
        renderer.post_transparent = Some(Box::new(GizmoRenderer::new(gpu, &assets)));

        let ui_renderer = Renderer::new(
            gpu,
            &assets,
            RendererConfig {
                scene: ui_scene(),
                shadows: false,
                forward: false,
                ..Default::default()
            },
        );

        Self {
            world: game_world,
            systems,
            temporary_systems: Default::default(),
            gpu_world_sync_systems: gpu_world_sync_systems(),
            renderer,
            ui_renderer,
            assets,
            user_id: player_id,
        }
    }
    #[ambient_profiling::function]
    pub fn on_frame(&mut self, gpu: &Gpu, target: &RenderTarget) {
        let _span = debug_span!("ClientGameState.on_frame").entered();

        self.world.next_frame();
        self.systems.run(&mut self.world, &FrameEvent);
        self.temporary_systems
            .retain_mut(|system| !(system.0)(&mut self.world));

        self.gpu_world_sync_systems
            .run(&mut self.world, &GpuWorldSyncEvent);
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GameState.render"),
            });
        let mut post_submit = Vec::new();
        tracing::trace!("Drawing world");
        self.renderer.render(
            gpu,
            &mut self.world,
            &mut encoder,
            &mut post_submit,
            RendererTarget::Target(target),
            Some(Color::rgba(0., 0., 0., 1.)),
        );

        tracing::trace!("Drawing ui");

        self.ui_renderer.render(
            gpu,
            &mut self.world,
            &mut encoder,
            &mut post_submit,
            RendererTarget::Target(target),
            None,
        );

        gpu.queue.submit(Some(encoder.finish()));
        for action in post_submit {
            action();
        }
    }
    /// Adds a temporary system; when it returns true it's removed
    pub fn add_temporary_system(
        &mut self,
        system: impl FnMut(&mut World) -> bool + Sync + Send + 'static,
    ) {
        self.temporary_systems.push(TempSystem(Box::new(system)));
    }

    pub fn proj_view(&self) -> Option<Mat4> {
        let camera = get_active_camera(&self.world, main_scene(), Some(&self.user_id))?;
        // This can only work client side, since project_view only exists there (which in turn requires the screen size)
        self.world.get(camera, projection_view()).ok()
    }
    pub fn view(&self) -> Option<Mat4> {
        let camera = get_active_camera(&self.world, main_scene(), Some(&self.user_id))?;
        // // This can only work client side, since project_view only exists there (which in turn requires the screen size)
        Some(self.world.get(camera, local_to_world()).ok()?.inverse())
    }

    pub fn center_screen_ray(&self) -> Ray {
        self.screen_ray(Vec2::ZERO)
    }
    pub fn screen_ray(&self, clip_space_pos: Vec2) -> Ray {
        let inv_proj_view = self.proj_view().unwrap_or(Mat4::IDENTITY).inverse();
        let a = inv_proj_view.project_point3(clip_space_pos.extend(1.));
        let b = inv_proj_view.project_point3(clip_space_pos.extend(0.9));
        let origin = a;
        let dir = (b - a).normalize();
        Ray { origin, dir }
    }
    pub fn clip_to_world_space(&self, p: Vec3) -> Vec3 {
        let inv_proj_view = self.proj_view().unwrap_or(Mat4::IDENTITY).inverse();
        inv_proj_view.project_point3(p)
    }
    pub fn world_to_clip_space(&self, p: Vec3) -> Vec3 {
        let proj_view = self.proj_view().unwrap_or(Mat4::IDENTITY);
        proj_view.project_point3(p)
    }
    pub fn clip_to_screen_space(&self, p: Vec3) -> Vec2 {
        let screen_size = *self.world.resource(window_physical_size());
        interpolate(
            p.xy(),
            vec2(-1., 1.),
            vec2(1., -1.),
            Vec2::ZERO,
            screen_size.as_vec2(),
        )
    }
    pub fn world_to_screen_space(&self, p: Vec3) -> Vec2 {
        self.clip_to_screen_space(self.world_to_clip_space(p))
    }

    pub fn is_master_client(&self) -> bool {
        let first = query((user_id(), player()))
            .iter(&self.world, None)
            .map(|(_, (id, _))| id.clone())
            .min();
        Some(&self.user_id) == first.as_ref()
    }
}
