use std::sync::Arc;

use ambient_gpu::{
    gpu::{Gpu, DEFAULT_SAMPLE_COUNT},
    shader_module::DEPTH_FORMAT,
    texture::{Texture, TextureView},
};
use glam::UVec2;
use wgpu::{TextureFormat, TextureViewDescriptor};

/// TODO: remove in favor of https://docs.rs/wgpu/latest/wgpu/enum.TextureFormat.html#method.add_srgb_suffix after upgrading to wgpu@0.15.2
pub(crate) fn to_linear_format(format: TextureFormat) -> TextureFormat {
    match format {
        TextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8Unorm,
        TextureFormat::Bgra8UnormSrgb => TextureFormat::Bgra8Unorm,
        TextureFormat::Bc1RgbaUnormSrgb => TextureFormat::Bc1RgbaUnorm,
        TextureFormat::Bc2RgbaUnormSrgb => TextureFormat::Bc2RgbaUnorm,
        TextureFormat::Bc3RgbaUnormSrgb => TextureFormat::Bc3RgbaUnorm,
        TextureFormat::Bc7RgbaUnormSrgb => TextureFormat::Bc7RgbaUnorm,
        TextureFormat::Etc2Rgb8UnormSrgb => TextureFormat::Etc2Rgb8Unorm,
        TextureFormat::Etc2Rgb8A1UnormSrgb => TextureFormat::Etc2Rgb8A1Unorm,
        TextureFormat::Etc2Rgba8UnormSrgb => TextureFormat::Etc2Rgba8Unorm,
        _ => format,
    }
}

#[derive(Debug)]
pub struct RenderTarget {
    pub depth_buffer: Arc<Texture>,
    pub depth_buffer_view: TextureView,
    pub depth_stencil_view: TextureView,
    pub color_buffer: Arc<Texture>,
    pub color_buffer_view: TextureView,
    pub normals_quat_buffer: Arc<Texture>,
    pub normals_quat_buffer_view: TextureView,
}
impl RenderTarget {
    pub fn new(gpu: Arc<Gpu>, size: UVec2, usage: Option<wgpu::TextureUsages>) -> Self {
        let usage = usage.unwrap_or(
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let sc_desc = gpu.sc_desc(size);
        let depth_buffer = Arc::new(Texture::new(
            gpu.clone(),
            &wgpu::TextureDescriptor {
                label: Some("RenderTarget.depth_buffer"),
                size: wgpu::Extent3d {
                    width: sc_desc.width,
                    height: sc_desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: DEFAULT_SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage,
                view_formats: &[],
            },
        ));
        let color_buffer = Arc::new(Texture::new(
            gpu.clone(),
            &wgpu::TextureDescriptor {
                label: Some("RenderTarget.color_buffer"),
                size: wgpu::Extent3d {
                    width: sc_desc.width,
                    height: sc_desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: sc_desc.format,
                usage,
                view_formats: &[],
            },
        ));
        let normals_buffer = Arc::new(Texture::new(
            gpu,
            &wgpu::TextureDescriptor {
                label: Some("RenderTarget.normals_quat_buffer"),
                size: wgpu::Extent3d {
                    width: sc_desc.width,
                    height: sc_desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: to_linear_format(sc_desc.format),
                usage,
                view_formats: &[],
            },
        ));
        Self {
            depth_buffer_view: depth_buffer.create_view(&TextureViewDescriptor {
                aspect: wgpu::TextureAspect::DepthOnly,
                ..Default::default()
            }),
            depth_stencil_view: depth_buffer.create_view(&TextureViewDescriptor {
                ..Default::default()
            }),
            depth_buffer,
            color_buffer_view: color_buffer.create_view(&Default::default()),
            color_buffer,
            normals_quat_buffer_view: normals_buffer.create_view(&Default::default()),
            normals_quat_buffer: normals_buffer,
        }
    }
}
