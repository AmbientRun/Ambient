use std::{borrow::Cow, sync::Arc};

use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    texture::{Texture, TextureView},
    texture_loaders::TextureFromUrl,
    wgsl_utils::wgsl_interpolate, sampler::SamplerKey,
};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    include_file,
};
use glam::{vec2, UVec2, Vec2};
use itertools::Itertools;
use rand::prelude::*;
use rand_pcg::Pcg64;
use wgpu::{
    util::DeviceExt, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages, TextureFormat,
    TextureViewDimension,
};

use super::BrushWGSL;
use crate::{wgsl_terrain_preprocess, OLD_CONTENT_SERVER_URL};

#[derive(Clone, Copy, Debug, Default)]
pub struct RaiseBrushConfig {
    pub params: RaiseBrushParams,
    pub seed: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RaiseBrushParams {
    pub brush: BrushWGSL,
    pub heightmap_world_position: Vec2,
    pub heightmap_world_texel_size: f32,
    pub layer: i32,

    pub _padding: UVec2,
}
impl Default for RaiseBrushParams {
    fn default() -> Self {
        Self {
            heightmap_world_position: Vec2::ZERO,
            brush: Default::default(),
            heightmap_world_texel_size: 0.,
            layer: 0,

            _padding: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct RaiseBrush {
    pipeline: wgpu::ComputePipeline,
    noise_texture: Arc<Texture>,
    noise_sampler: Arc<wgpu::Sampler>,
}
impl RaiseBrush {
    pub async fn new(assets: AssetCache) -> Self {
        let gpu = GpuKey.get(&assets);
        let shader = [
            &wgsl_interpolate() as &str,
            &include_file!("snoise.wgsl"),
            &include_file!("brush.wgsl"),
            &wgsl_terrain_preprocess(include_file!("raise.wgsl")),
        ]
        .join("\n");
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("RaiseBrush.shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader)),
        });

        let pipeline = gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::ReadWrite,
                                format: TextureFormat::R32Float,
                                view_dimension: TextureViewDimension::D2Array,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 3,
                            visibility: ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                })],
                push_constant_ranges: &[],
            })),
            module: &shader,
            entry_point: "main",
        });
        Self {
            pipeline,
            noise_texture: TextureFromUrl {
                url: AbsAssetUrl::parse(format!(
                    "{OLD_CONTENT_SERVER_URL}assets/models/{}",
                    "ArtStationSurfaces/VFX-HQ-Seamless-Noise-Pack-Vol1/Noise_002.png"
                ))
                .unwrap(),
                format: wgpu::TextureFormat::Rgba8Unorm,
            }
            .get(&assets)
            .await
            .unwrap(),
            noise_sampler: SamplerKey::LINEAR_CLAMP_TO_EDGE.get(&assets),
        }
    }
    pub fn run(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder, heightmap: &TextureView, size: UVec2, config: &RaiseBrushConfig) {
        let param_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameter Buffer"),
            contents: bytemuck::cast_slice(&[config.params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut rng = Pcg64::seed_from_u64(config.seed);
        let offsets = (0..12).map(|_| vec2(rng.gen::<f32>() * 1000., rng.gen::<f32>() * 1000.)).collect_vec();

        let offsets = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Offsets"),
            contents: bytemuck::cast_slice(&offsets),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(heightmap) },
                wgpu::BindGroupEntry { binding: 1, resource: param_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: offsets.as_entire_binding() },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.noise_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.noise_sampler) },
            ],
        });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(size.x, size.y, 1);
    }
}
