use std::sync::Arc;

use ambient_core::{camera::Camera, main_scene, player::local_user_id, transform::*};
use ambient_ecs::{ArchetypeFilter, World};
use ambient_gpu::{
    gpu::GpuKey,
    mesh_buffer::MeshBuffer,
    shader_module::DEPTH_FORMAT,
    texture::{Texture, TextureView},
};
use ambient_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use itertools::Itertools;
use smallvec::SmallVec;
use wgpu::DepthBiasState;

use super::{
    cast_shadows, get_active_sun, FSMain, RendererCollectState, RendererResources,
    ShadowAndUIGlobals, TreeRenderer, TreeRendererConfig, MAX_SHADOW_CASCADES,
};
use crate::{bind_groups::BindGroups, default_sun_direction, PostSubmitFunc, RendererConfig};

pub struct ShadowsRenderer {
    renderer: TreeRenderer,
    cascades: Vec<ShadowCascade>,
    pub shadow_texture: Arc<Texture>,
    config: RendererConfig,
    pub shadow_view: TextureView,
}

impl std::fmt::Debug for ShadowsRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShadowsRenderer")
            .field("config", &self.config)
            .field("shadow_view", &self.shadow_view)
            .finish()
    }
}

impl ShadowsRenderer {
    pub fn new(
        assets: AssetCache,
        renderer_resources: RendererResources,
        config: RendererConfig,
    ) -> Self {
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
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
        ));

        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });

        Self {
            renderer: TreeRenderer::new(TreeRendererConfig {
                gpu,
                assets: assets.clone(),
                renderer_config: config.clone(),
                targets: vec![],
                filter: ArchetypeFilter::new()
                    .incl(main_scene())
                    .incl(cast_shadows()),
                renderer_resources: renderer_resources.clone(),
                fs_main: FSMain::Shadow,
                opaque_only: false,
                depth_stencil: true,
                cull_mode: Some(wgpu::Face::Back),
                depth_bias: DepthBiasState {
                    constant: -2,
                    slope_scale: -1.5,
                    clamp: 0.0,
                },
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
                        array_layer_count: Some(1),
                    }),
                    globals: ShadowAndUIGlobals::new(
                        assets.clone(),
                        renderer_resources.globals_layout.clone(),
                    ),
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

    #[ambient_profiling::function]
    pub fn update(&mut self, world: &mut World) {
        let main_camera =
            Camera::get_active(world, main_scene(), world.resource_opt(local_user_id()))
                .unwrap_or_default();

        let sun_direction = if let Some(sun) = get_active_sun(world, main_scene()) {
            get_world_rotation(world, sun).unwrap().mul_vec3(Vec3::X)
        } else {
            default_sun_direction()
        };

        self.renderer.update(world);

        for (i, cascade) in self.cascades.iter_mut().enumerate() {
            ambient_profiling::scope!("Shadow cascade update");
            let new_camera = main_camera.create_snapping_shadow_camera(
                sun_direction,
                i as u32,
                self.config.shadow_cascades,
                self.config.shadow_map_resolution,
            );
            cascade
                .globals
                .update(world, main_scene(), new_camera.projection_view());
            cascade.camera = new_camera;
            cascade.collect_state.set_camera(i as u32 + 1);
        }
    }

    pub fn stats(&self) -> String {
        let shadow_entities: usize =
            self.renderer.n_entities() * self.config.shadow_cascades as usize;
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

impl ShadowsRenderer {
    pub fn render<'a>(
        &'a mut self,
        mesh_buffer: &MeshBuffer,
        encoder: &mut wgpu::CommandEncoder,
        bind_groups: &BindGroups<'a>,
        post_submit: &mut Vec<PostSubmitFunc>,
    ) {
        for (i, cascade) in self.cascades.iter_mut().enumerate() {
            ambient_profiling::scope!("Shadow dynamic render");
            self.renderer.run_collect(
                encoder,
                post_submit,
                bind_groups.mesh_meta,
                bind_groups.entities,
                &mut cascade.collect_state,
            );
            let label = format!("Shadow cascade {i}");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&label),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &cascade.dynamic_target,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            let globals = cascade.globals.create_bind_group(mesh_buffer);

            render_pass.set_index_buffer(
                mesh_buffer.index_buffer.buffer().slice(..),
                wgpu::IndexFormat::Uint32,
            );
            self.renderer.render(
                &mut render_pass,
                &cascade.collect_state,
                &BindGroups {
                    globals,
                    ..*bind_groups
                },
            );
            {
                ambient_profiling::scope!("Drop render pass");
                drop(render_pass);
            }
        }
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
