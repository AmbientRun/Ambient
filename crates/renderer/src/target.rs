use std::sync::Arc;

use elements_gpu::{
    gpu::{Gpu, DEFAULT_SAMPLE_COUNT}, texture::{Texture, TextureView}
};
use glam::UVec2;

#[derive(Debug)]
pub struct RenderTarget {
    pub depth_buffer: Arc<Texture>,
    pub depth_buffer_view: TextureView,
    pub screen_buffer: Arc<Texture>,
    pub screen_buffer_view: TextureView,
}
impl RenderTarget {
    pub fn new(gpu: Arc<Gpu>, size: UVec2, usage: Option<wgpu::TextureUsages>) -> Self {
        let usage =
            usage.unwrap_or(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC);
        let sc_desc = gpu.sc_desc(size);
        let depth_buffer = Arc::new(Texture::new(
            gpu.clone(),
            &wgpu::TextureDescriptor {
                label: Some("RenderTarget.depth_buffer"),
                size: wgpu::Extent3d { width: sc_desc.width, height: sc_desc.height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: DEFAULT_SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage,
            },
        ));
        let screen_buffer = Arc::new(Texture::new(
            gpu,
            &wgpu::TextureDescriptor {
                label: Some("RenderTarget.screen_buffer"),
                size: wgpu::Extent3d { width: sc_desc.width, height: sc_desc.height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: sc_desc.format,
                usage,
            },
        ));
        Self {
            depth_buffer_view: depth_buffer.create_view(&wgpu::TextureViewDescriptor::default()),
            depth_buffer,
            screen_buffer_view: screen_buffer.create_view(&wgpu::TextureViewDescriptor::default()),
            screen_buffer,
        }
    }
}
