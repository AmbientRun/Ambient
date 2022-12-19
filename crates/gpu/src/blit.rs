use std::{borrow::Cow, sync::Arc};

use elements_std::asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt};

use super::gpu::{Gpu, GpuKey};

#[derive(Debug, Clone)]
pub struct BlitterKey {
    pub format: wgpu::ColorTargetState,
    pub linear: bool,
}
impl SyncAssetKey<Arc<Blitter>> for BlitterKey {
    fn load(&self, assets: AssetCache) -> Arc<Blitter> {
        let gpu = GpuKey.get(&assets);
        Arc::new(Blitter::new(gpu, self))
    }
}

pub struct Blitter {
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    gpu: Arc<Gpu>,
}
impl Blitter {
    pub fn new(gpu: Arc<Gpu>, conf: &BlitterKey) -> Self {
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blitter.shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("blit.wgsl"))),
        });

        let pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blitter.pipeline"),
            layout: None,
            vertex: wgpu::VertexState { module: &shader, entry_point: "vs_main", buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "fs_main", targets: &[Some(conf.format.clone())] }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() },
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
            min_filter: if conf.linear { wgpu::FilterMode::Linear } else { wgpu::FilterMode::Nearest },
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self { pipeline, sampler, gpu }
    }
    pub fn run(&self, encoder: &mut wgpu::CommandEncoder, source: &wgpu::TextureView, target: &wgpu::TextureView) {
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);

        let bind_group = self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(source) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
            label: None,
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::WHITE), store: true },
            })],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..4, 0..1);
    }
}
