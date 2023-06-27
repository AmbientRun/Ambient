use std::sync::Arc;

use ambient_core::{
    gpu_components,
    gpu_ecs::{ComponentToGpuSystem, GpuComponentFormat, GpuWorldSyncEvent},
};
use ambient_ecs::{copy_component_recursive, ArchetypeFilter, Component, SystemGroup, World};
use ambient_gpu::{
    gpu::Gpu,
    mesh_buffer::MeshBuffer,
    shader_module::{BindGroupDesc, GraphicsPipeline, GraphicsPipelineInfo, Shader},
    texture::Texture,
};
use ambient_std::{asset_cache::AssetCache, include_file};
use wgpu::{BindGroupLayoutEntry, BindingType, PrimitiveTopology, ShaderStages};

use super::{
    FSMain, RendererCollectState, RendererResources, RendererTarget, ShaderModule, TreeRenderer,
    TreeRendererConfig,
};
use crate::{bind_groups::BindGroups, PostSubmitFunc, RendererConfig};

pub use ambient_ecs::generated::components::core::rendering::{outline, outline_recursive};

gpu_components! {
    outline() => outline: GpuComponentFormat::Vec4,
}

pub struct OutlinesConfig {
    pub scene: Component<()>,
    pub renderer_resources: RendererResources,
}

pub struct Outlines {
    outlines: Arc<Texture>,
    pipeline: GraphicsPipeline,
    renderer: TreeRenderer,
    collect_state: RendererCollectState,
    _config: OutlinesConfig,
}

const OUTLINES_BIND_GROUP: &str = "OUTLINES_BIND_GROUP";

fn get_outlines_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }],
        label: OUTLINES_BIND_GROUP.into(),
    }
}

impl Outlines {
    pub fn new(
        gpu: &Gpu,
        assets: &AssetCache,
        config: OutlinesConfig,
        renderer_config: RendererConfig,
    ) -> Self {
        let shader = Shader::new(
            assets,
            "Outlines",
            &[OUTLINES_BIND_GROUP],
            &ShaderModule::new("outlines", include_file!("outlines.wgsl"))
                .with_binding_desc(get_outlines_layout()),
        )
        .unwrap();

        let pipeline = shader.to_pipeline(
            gpu,
            GraphicsPipelineInfo {
                targets: &[Some(gpu.swapchain_format().into())],
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
        );

        Self {
            outlines: Self::create_outline_texture(
                gpu,
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            ),
            pipeline,
            collect_state: RendererCollectState::new(gpu),
            renderer: TreeRenderer::new(
                gpu,
                TreeRendererConfig {
                    renderer_config,
                    targets: vec![Some(wgpu::ColorTargetState {
                        format: Outlines::FORMAT,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                    filter: ArchetypeFilter::new().incl(config.scene).incl(outline()),
                    renderer_resources: config.renderer_resources.clone(),
                    fs_main: FSMain::Outline,
                    opaque_only: false,
                    depth_stencil: false,
                    cull_mode: Some(wgpu::Face::Back),
                    depth_bias: Default::default(),
                },
            ),
            _config: config,
        }
    }

    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    fn create_outline_texture(gpu: &Gpu, size: wgpu::Extent3d) -> Arc<Texture> {
        Arc::new(Texture::new(
            gpu,
            &wgpu::TextureDescriptor {
                label: Some("Renderer.outlines"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        gpu: &Gpu,
        assets: &AssetCache,
        world: &mut World,
        encoder: &mut wgpu::CommandEncoder,
        post_submit: &mut Vec<PostSubmitFunc>,
        target: &RendererTarget,
        bind_groups: &BindGroups,
        mesh_buffer: &MeshBuffer,
    ) {
        let bind_group_layout = self.pipeline.pipeline().get_bind_group_layout(0);

        if self.outlines.size != target.size() {
            self.outlines = Self::create_outline_texture(gpu, target.size());
        }
        let outlines = self.outlines.create_view(&Default::default());

        self.collect_state.set_camera(gpu, 0);
        self.renderer.update(gpu, assets, world);
        self.renderer.run_collect(
            gpu,
            assets,
            encoder,
            post_submit,
            bind_groups.mesh_meta,
            bind_groups.entities,
            &mut self.collect_state,
        );

        {
            ambient_profiling::scope!("Outlines stencil");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Outlines"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &outlines,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_index_buffer(
                mesh_buffer.index_buffer.buffer().slice(..),
                wgpu::IndexFormat::Uint32,
            );

            self.renderer.render(
                &mut render_pass,
                &self.collect_state,
                bind_groups,
                target.size(),
            );
            {
                ambient_profiling::scope!("Drop render pass");
                drop(render_pass);
            }
        }

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&outlines),
            }],
            label: None,
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.color(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(self.pipeline.pipeline());
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..4, 0..1);
    }
    pub fn dump(&self, f: &mut dyn std::io::Write) {
        self.renderer.dump(f);
    }
}

pub fn systems() -> SystemGroup {
    copy_component_recursive("outlines", outline_recursive(), outline())
}

pub fn gpu_world_systems() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "outlines/gpu_world_update",
        vec![Box::new(ComponentToGpuSystem::new(
            GpuComponentFormat::Vec4,
            outline(),
            gpu_components::outline(),
        ))],
    )
}
