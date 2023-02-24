use std::sync::Arc;

use ambient_app::{gpu_world_sync_systems, world_instance_resources, world_instance_systems, AppResources};
use ambient_core::{
    camera::{get_active_camera, projection_view},
    gpu_ecs::GpuWorldSyncEvent,
    main_scene,
    transform::local_to_world,
    window_physical_size,
};
use ambient_ecs::{components, query, EntityData, FrameEvent, System, SystemGroup, World};
use ambient_gizmos::render::GizmoRenderer;
use ambient_gpu::gpu::GpuKey;
use ambient_renderer::{RenderTarget, Renderer, RendererConfig, RendererTarget};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    color::Color,
    math::interpolate,
    shapes::Ray,
};
use glam::{vec2, Mat4, Vec2, Vec3, Vec3Swizzles};

use crate::{player, user_id};

components!("rendering", {
    game_screen_render_target: Arc<RenderTarget>,
});

#[derive(Debug)]
/// Holds the physical world
pub struct ClientGameState {
    pub world: World,
    systems: SystemGroup,
    temporary_systems: Vec<TempSystem>,
    gpu_world_sync_systems: SystemGroup<GpuWorldSyncEvent>,
    pub renderer: Renderer,
    assets: AssetCache,
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
        world: &mut World,
        assets: AssetCache,
        player_id: String,
        render_target: Arc<RenderTarget>,
        client_systems: SystemGroup,
        client_resources: EntityData,
    ) -> Self {
        let mut game_world = World::new("client_game_world");
        let local_resources = world_instance_resources(AppResources::from_world(world))
            .set(crate::local_user_id(), player_id.clone())
            .set(game_screen_render_target(), render_target)
            .append(client_resources);
        game_world.add_components(game_world.resource_entity(), local_resources).unwrap();

        let systems = SystemGroup::new("game", vec![Box::new(client_systems), Box::new(world_instance_systems(true))]);
        let mut renderer =
            Renderer::new(world, assets.clone(), RendererConfig { scene: main_scene(), shadows: true, ..Default::default() });
        renderer.post_transparent = Some(Box::new(GizmoRenderer::new(&assets)));

        Self {
            world: game_world,
            systems,
            temporary_systems: Default::default(),
            gpu_world_sync_systems: gpu_world_sync_systems(),
            renderer,
            assets,
            user_id: player_id,
        }
    }
    #[profiling::function]
    pub fn on_frame(&mut self, target: &RenderTarget) {
        self.systems.run(&mut self.world, &FrameEvent);
        self.temporary_systems.retain_mut(|system| !(system.0)(&mut self.world));
        self.gpu_world_sync_systems.run(&mut self.world, &GpuWorldSyncEvent);
        let gpu = GpuKey.get(&self.assets);
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("GameState.render") });
        let mut post_submit = Vec::new();
        self.renderer.render(
            &mut self.world,
            &mut encoder,
            &mut post_submit,
            RendererTarget::Target(target),
            Some(Color::rgba(0., 0., 0., 1.)),
        );
        gpu.queue.submit(Some(encoder.finish()));
        for action in post_submit {
            action();
        }
    }
    /// Adds a temporary system; when it returns true it's removed
    pub fn add_temporary_system(&mut self, system: impl FnMut(&mut World) -> bool + Sync + Send + 'static) {
        self.temporary_systems.push(TempSystem(Box::new(system)));
    }

    pub fn proj_view(&self) -> Option<Mat4> {
        let camera = get_active_camera(&self.world, main_scene())?;
        // This can only work client side, since project_view only exists there (which in turn requires the screen size)
        self.world.get(camera, projection_view()).ok()
    }
    pub fn view(&self) -> Option<Mat4> {
        let camera = get_active_camera(&self.world, main_scene())?;
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
        interpolate(p.xy(), vec2(-1., 1.), vec2(1., -1.), Vec2::ZERO, screen_size.as_vec2())
    }
    pub fn world_to_screen_space(&self, p: Vec3) -> Vec2 {
        self.clip_to_screen_space(self.world_to_clip_space(p))
    }

    pub fn is_master_client(&self) -> bool {
        let first = query((user_id(), player())).iter(&self.world, None).map(|(_, (id, _))| id.clone()).min();
        Some(&self.user_id) == first.as_ref()
    }
}
