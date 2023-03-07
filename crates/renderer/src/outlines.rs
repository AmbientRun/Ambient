use std::sync::Arc;

use ambient_core::{
    gpu_components,
    gpu_ecs::{ComponentToGpuSystem, GpuComponentFormat, GpuWorldSyncEvent},
    hierarchy::children,
};
use ambient_ecs::{components, query, ArchetypeFilter, Component, Description, Name, Networked, Store, SystemGroup, World};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    mesh_buffer::MeshBuffer,
    shader_module::{BindGroupDesc, GraphicsPipeline, GraphicsPipelineInfo, Shader},
    texture::Texture,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    include_file,
};
use glam::Vec4;
use wgpu::{BindGroup, BindGroupLayoutEntry, BindingType, PrimitiveTopology, ShaderStages};

use super::{FSMain, RendererCollectState, RendererResources, RendererTarget, ShaderModule, TreeRenderer, TreeRendererConfig};
use crate::RendererConfig;

components!("rendering", {
    @[
        Networked, Store,
        Name["Outline"],
        Description["If attached, this entity will be rendered with an outline with the color specified."]
    ]
    outline: Vec4,
    @[
        Networked, Store,
        Name["Outline (recursive)"],
        Description["If attached, this entity and all of its children will be rendered with an outline with the color specified.\nYou do not need to attach `outline` if you have attached `outline_recursive`."]
    ]
    outline_recursive: Vec4,
});
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
    gpu: Arc<Gpu>,
}
impl Outlines {
    pub fn new(assets: &AssetCache, config: OutlinesConfig, renderer_config: RendererConfig) -> Self {
        let gpu = GpuKey.get(assets);

        let shader = Shader::from_modules(
            assets,
            "Outlines",
            &[ShaderModule::new(
                "Outlines",
                include_file!("outlines.wgsl"),
                vec![BindGroupDesc {
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
                    label: "OUTLINES_BIND_GROUP".into(),
                }
                .into()], // vec![BindGroupDesc { entries: vec![BindGroupEntry { binding: 0, resource: ) }], label: todo!() }],
            )],
        );

        let pipeline = shader.to_pipeline(
            &gpu,
            GraphicsPipelineInfo {
                targets: &[Some(gpu.swapchain_format().into())],
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
        );

        Self {
            outlines: Self::create_outline_texture(gpu.clone(), wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 }),
            pipeline,
            collect_state: RendererCollectState::new(assets),
            renderer: TreeRenderer::new(TreeRendererConfig {
                gpu: gpu.clone(),
                assets: assets.clone(),
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
            }),
            _config: config,
            gpu,
        }
    }

    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    fn create_outline_texture(gpu: Arc<Gpu>, size: wgpu::Extent3d) -> Arc<Texture> {
        Arc::new(Texture::new(
            gpu,
            &wgpu::TextureDescriptor {
                label: Some("Renderer.outlines"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            },
        ))
    }

    pub fn render(
        &mut self,
        world: &mut World,
        encoder: &mut wgpu::CommandEncoder,
        post_submit: &mut Vec<Box<dyn FnOnce() + Send + Send>>,
        target: &RendererTarget,
        binds: &[(&str, &BindGroup)],
        mesh_buffer: &MeshBuffer,
    ) {
        let bind_group_layout = self.pipeline.pipeline().get_bind_group_layout(0);

        if self.outlines.size != target.size() {
            self.outlines = Self::create_outline_texture(self.gpu.clone(), target.size());
        }
        let outlines = self.outlines.create_view(&Default::default());

        self.collect_state.set_camera(0);
        self.renderer.update(world);
        self.renderer.run_collect(encoder, post_submit, binds[0].1, binds[1].1, &mut self.collect_state);

        {
            profiling::scope!("Outlines stencil");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Outlines"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &outlines,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: true },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_index_buffer(mesh_buffer.index_buffer.buffer().slice(..), wgpu::IndexFormat::Uint32);

            // TODO maybe not necessary
            // for (name, group) in binds {
            //     self.pipeline.bind(&mut render_pass, name, *group);
            // }

            self.renderer.render(&mut render_pass, &self.collect_state, binds);
            {
                profiling::scope!("Drop render pass");
                drop(render_pass);
            }
        }

        let bind_group = self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&outlines) }],
            label: None,
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.color(),
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
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
    SystemGroup::new(
        "outlines",
        vec![
            query((outline_recursive().changed(),)).to_system(|q, world, qs, _| {
                for (id, (val,)) in q.collect_cloned(world, qs) {
                    world.add_component(id, outline(), val).ok();
                }
            }),
            query((outline_recursive(),)).despawned().to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    world.remove_component(id, outline()).ok();
                }
            }),
            query((outline_recursive(), children().changed())).to_system(|q, world, qs, _| {
                for (_, (val, childs)) in q.collect_cloned(world, qs) {
                    for c in childs {
                        world.add_component(c, outline_recursive(), val).ok();
                    }
                }
            }),
            query((outline_recursive(), children())).despawned().to_system(|q, world, qs, _| {
                for (_, (_, childs)) in q.collect_cloned(world, qs) {
                    for c in childs {
                        world.remove_component(c, outline_recursive()).ok();
                    }
                }
            }),
        ],
    )
}

pub fn gpu_world_systems() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "outlines/gpu_world_update",
        vec![Box::new(ComponentToGpuSystem::new(GpuComponentFormat::Vec4, outline(), gpu_components::outline()))],
    )
}
