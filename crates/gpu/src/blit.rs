use std::sync::Arc;

use ambient_std::asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt};
use wgpu::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, FilterMode, PipelineLayoutDescriptor,
    ShaderStages, TextureSampleType,
};

use crate::shader_module::{Shader, ShaderIdent, ShaderModule};

use super::gpu::{Gpu, GpuKey};

#[derive(Debug, Clone)]
pub struct BlitterKey {
    pub format: wgpu::ColorTargetState,
    pub min_filter: FilterMode,
    pub gamma_correction: Option<f32>,
}

impl SyncAssetKey<Arc<Blitter>> for BlitterKey {
    fn load(&self, assets: AssetCache) -> Arc<Blitter> {
        Arc::new(Blitter::new(&assets, self))
    }
}

pub struct Blitter {
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    gpu: Arc<Gpu>,
}
impl Blitter {
    pub fn new(assets: &AssetCache, conf: &BlitterKey) -> Self {
        let gpu = GpuKey.get(assets);

        log::debug!("Creating blitter: {conf:#?}");
        let colorspace = if let Some(gamma) = conf.gamma_correction {
            let inv_gamma = 1.0 / gamma;
            format!("vec4<f32>(pow(color.xyz, vec3<f32>({inv_gamma})), color.w)")
        } else {
            "color".to_string()
        };

        let shader = Shader::new(
            assets,
            "blitter",
            &[],
            &ShaderModule::new("blitter", include_str!("blit.wgsl"))
                .with_ident(ShaderIdent::raw("COLORSPACE_EXPR", colorspace)),
        )
        .unwrap();

        let bind_group_layout = gpu
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("blitter.bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("blitter.layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        log::debug!("Setting up blitter");
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Blitter.pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(conf.format.clone())],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blitter.sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: conf.min_filter,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            sampler,
            gpu,
        }
    }

    pub fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
    ) {
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);

        let bind_group = self
            .gpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: None,
            });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..4, 0..1);
    }
}
