use std::borrow::Cow;

use ambient_gpu::{gpu::Gpu, texture::TextureView};
use ambient_std::include_file;
use glam::{IVec2, UVec2, Vec2};
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use crate::wgsl_terrain_preprocess;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ThermalErosionConfig {
    pub params: ThermalErosionParams,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ThermalErosionParams {
    pub heightmap_world_position: Vec2,
    pub heightmap_world_size: Vec2,
    pub heightmap_texture_size: IVec2,
    pub brush_position: Vec2,
    pub brush_radius: f32,
    pub frame: i32,
    pub _padding: IVec2,
}

pub struct ThermalErosionCompute {
    pipeline: wgpu::ComputePipeline,
}
impl ThermalErosionCompute {
    pub fn new(gpu: &Gpu) -> Self {
        Self { pipeline: Self::create_pipeline(gpu) }
    }
    fn create_pipeline(gpu: &Gpu) -> wgpu::ComputePipeline {
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ThermalErosionCompute.shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(wgsl_terrain_preprocess(include_file!("thermal_erosion.wgsl")))),
        });

        gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::ReadWrite,
                                format: wgpu::TextureFormat::R32Float,
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
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
        })
    }
    pub fn run(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder, texture: &TextureView, size: UVec2, config: &ThermalErosionConfig) {
        let param_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameter Buffer"),
            contents: bytemuck::cast_slice(&[config.params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(texture) },
                wgpu::BindGroupEntry { binding: 1, resource: param_buffer.as_entire_binding() },
            ],
        });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(size.x, size.y, 1);
    }
}
