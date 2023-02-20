use std::{borrow::Cow, sync::Arc};

use ambient_gpu::{gpu::Gpu, texture::Texture, wgsl_utils::wgsl_interpolate};
use ambient_std::include_file;
use glam::{vec2, Vec2};
use itertools::Itertools;
use rand::prelude::*;
use rand_pcg::Pcg64;
use wgpu::{
    util::DeviceExt, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages, TextureFormat,
    TextureViewDimension,
};

use crate::wgsl_terrain_preprocess;

#[derive(Clone, Copy, Debug, Default)]
pub struct InitGroundConfig {
    pub params: InitGroundParams,
    pub seed: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InitGroundParams {
    pub heightmap_world_position: Vec2,
    pub heightmap_world_size: Vec2,
}
impl Default for InitGroundParams {
    fn default() -> Self {
        Self { heightmap_world_position: Vec2::ZERO, heightmap_world_size: Vec2::ZERO }
    }
}

pub struct InitGroundBrush {
    pipeline: wgpu::ComputePipeline,
}
impl InitGroundBrush {
    pub fn new(gpu: &Gpu) -> Self {
        let shader = [&wgsl_interpolate() as &str, &include_file!("snoise.wgsl"), &include_file!("init.wgsl")].join("\n");
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GenerateTerrain.shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(wgsl_terrain_preprocess(shader))),
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
                    ],
                })],
                push_constant_ranges: &[],
            })),
            module: &shader,
            entry_point: "main",
        });
        Self { pipeline }
    }
    pub fn run(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder, heightmap: &Arc<Texture>, config: &InitGroundConfig) {
        let param_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameter Buffer"),
            contents: bytemuck::cast_slice(&[config.params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut rng = Pcg64::seed_from_u64(config.seed);
        let octaves = 9;
        let offsets = (0..octaves).map(|_| vec2(rng.gen::<f32>() * 1000., rng.gen::<f32>() * 1000.)).collect_vec();

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
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&heightmap.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry { binding: 1, resource: param_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: offsets.as_entire_binding() },
            ],
        });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(heightmap.size.width, heightmap.size.height, 1);
    }
}
