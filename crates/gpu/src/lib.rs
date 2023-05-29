pub mod blit;
pub mod fill;
pub mod gpu;
pub mod gpu_run;
pub mod mesh_buffer;
pub mod mipmap;
pub mod multi_buffer;
pub mod sampler;
pub mod settings;
pub mod shader_module;
pub mod std_assets;
pub mod texture;
pub mod texture_loaders;
pub mod typed_buffer;
pub mod wgsl_utils;

pub fn texture_format_to_wgsl_storage_format(format: wgpu::TextureFormat) -> &'static str {
    match format {
        wgpu::TextureFormat::R32Float => "r32float",
        wgpu::TextureFormat::Rgba32Float => "rgba32float",
        _ => panic!("Unsupported texture storage format"),
    }
}
