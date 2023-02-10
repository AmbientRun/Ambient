use std::{num::NonZeroU32, sync::Arc};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use itertools::Itertools;
use kiwi_core::{camera::Camera, gpu_ecs::ENTITIES_BIND_GROUP, main_scene, transform::*};
use kiwi_ecs::{ArchetypeFilter, World};
use kiwi_gpu::{
    gpu::GpuKey, mesh_buffer::MeshBuffer, texture::{Texture, TextureView}
};
use kiwi_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use smallvec::SmallVec;
use wgpu::DepthBiasState;

use super::{
    cast_shadows, get_active_sun, FSMain, RendererCollectState, RendererResources, RendererSettings, ShadowAndUIGlobals, TreeRenderer, TreeRendererConfig, GLOBALS_BIND_GROUP, MAX_SHADOW_CASCADES, RESOURCES_BIND_GROUP
};

pub struct ShadowsRenderer {
    renderer: TreeRenderer,
    cascades: Vec<ShadowCascade>,
    pub shadow_texture: Arc<Texture>,
    config: RendererSettings,
    pub shadow_view: TextureView,
}

impl ShadowsRenderer {
    pub fn new(assets: AssetCache, renderer_resources: RendererResources, config: RendererSettings) -> Self {
        let gpu = GpuKey.get(&assets);

        let shadow_texture = Arc::new(Texture::new(
            gpu.clone(),
            &wgpu::TextureDescriptor {
                label: Some("Renderer.shadow_texture"),
                size: wgpu::Extent3d {
                    width: config.shadow_map_resolution,
                    height: config.shadow_map_resolution,
                    depth_or_array_layers: config.shadow_cascades,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
            },
        ));
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            renderer: TreeRenderer::new(TreeRendererConfig {
                gpu,
                targets: vec![],
                filter: ArchetypeFilter::new().incl(main_scene()).incl(cast_shadows()),
                renderer_resources: renderer_resources.clone(),
                fs_main: FSMain::Shadow,
                opaque_only: false,
                depth_stencil: true,
                cull_mode: Some(wgpu::Face::Front),
                depth_bias: DepthBiasState { constant: -2, slope_scale: -1.5, clamp: 0.0 },
            }),
            cascades: (0..config.shadow_cascades)
                .map(|i| ShadowCascade {
                    dynamic_target: shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("Renderer.shadow_target_views"),
                        format: None,
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: i,
                        array_layer_count: NonZeroU32::new(1),
                    }),
                    globals: ShadowAndUIGlobals::new(assets.clone(), renderer_resources.globals_layout.clone()),
                    camera: Camera::default(),
                    collect_state: RendererCollectState::new(&assets),
                })
                .collect_vec(),
            shadow_texture,
            shadow_view,
            config,
        }
    }
    pub fn get_cameras(&self) -> SmallVec<[ShadowCameraData; MAX_SHADOW_CASCADES as usize]> {
        self.cascades.iter().map(|v| (&v.camera).into()).collect()
    }
    pub fn n_cascades(&self) -> usize {
        self.cascades.len()
    }
    #[profiling::function]
    pub fn run(
        &mut self,
        world: &mut World,
        encoder: &mut wgpu::CommandEncoder,
        post_submit: &mut Vec<Box<dyn FnOnce() + Send + Send>>,
        resources_bind_group: &wgpu::BindGroup,
        entities_bind_group: &wgpu::BindGroup,
        mesh_buffer: &MeshBuffer,
    ) {
        let main_camera = Camera::get_active(world, main_scene()).unwrap_or_default();

        let mut sun_direction = Vec3::ONE;
        if let Some(sun) = get_active_sun(world, main_scene()) {
            sun_direction = get_world_rotation(world, sun).unwrap().mul_vec3(Vec3::X);
        }
        sun_direction = sun_direction.normalize();

        self.renderer.update(world);

        for (i, cascade) in self.cascades.iter_mut().enumerate() {
            profiling::scope!("Shadow cascade update");
            let new_camera = main_camera.create_snapping_shadow_camera(
                sun_direction,
                i as u32,
                self.config.shadow_cascades,
                self.config.shadow_map_resolution,
            );
            cascade.globals.update(world, main_scene(), new_camera.projection_view());
            cascade.camera = new_camera;
            cascade.collect_state.set_camera(i as u32 + 1);
        }

        for (i, cascade) in self.cascades.iter_mut().enumerate() {
            profiling::scope!("Shadow dynamic render");
            self.renderer.run_collect(encoder, post_submit, resources_bind_group, entities_bind_group, &mut cascade.collect_state);
            let label = format!("Shadow cascade {i}");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&label),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &cascade.dynamic_target,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: true }),
                    stencil_ops: None,
                }),
            });
            let globals = &cascade.globals.bind_group;
            render_pass.set_index_buffer(mesh_buffer.index_buffer.buffer().slice(..), wgpu::IndexFormat::Uint32);
            self.renderer.render(
                &mut render_pass,
                &cascade.collect_state,
                &[(GLOBALS_BIND_GROUP, globals), (RESOURCES_BIND_GROUP, resources_bind_group), (ENTITIES_BIND_GROUP, entities_bind_group)],
            );
            {
                profiling::scope!("Drop render pass");
                drop(render_pass);
            }
        }
    }

    pub fn stats(&self) -> String {
        let shadow_entities: usize = self.renderer.n_entities() * self.config.shadow_cascades as usize;
        let shadow_nodes: usize = self.renderer.n_nodes();
        format!("shadow: {shadow_entities}/{shadow_nodes}")
    }

    pub fn dump(&self, f: &mut dyn std::io::Write) {
        // for (i, shadow) in self.cascades.iter().enumerate() {
        writeln!(f, "  shadow").ok();
        self.renderer.dump(f);
        // }
    }
}

struct ShadowCascade {
    dynamic_target: TextureView,
    globals: ShadowAndUIGlobals,
    camera: Camera,
    collect_state: RendererCollectState,
}

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct ShadowCameraData {
    pub viewproj: Mat4,
    pub far: f32,
    pub near: f32,
    _padding: [f32; 2],
}

impl From<&Camera> for ShadowCameraData {
    fn from(v: &Camera) -> Self {
        Self {
            viewproj: v.projection_view(),
            far: v.projection.far().unwrap_or(1e6),
            near: v.projection.near(),
            _padding: Default::default(),
        }
    }
}
