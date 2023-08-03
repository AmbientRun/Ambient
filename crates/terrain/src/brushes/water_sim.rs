use std::borrow::Cow;

use ambient_editor_derive::ElementEditor;
use ambient_gpu::{gpu::Gpu, texture::TextureView};
use ambient_native_std::include_file;
use glam::UVec2;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use crate::wgsl_terrain_preprocess;

#[derive(Clone, Debug, Serialize, Deserialize, ElementEditor, Default)]
pub struct WaterSimConfig {
    pub params: WaterSimParams,
}

#[repr(C)]
#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, ElementEditor, bytemuck::Pod, bytemuck::Zeroable,
)]
pub struct WaterSimParams {
    #[editor(slider, min = 0., max = 10.)]
    pub gravity: f32,
    pub frame: i32,
}
impl Default for WaterSimParams {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            frame: 0,
        }
    }
}

pub struct WaterSimCompute {
    rain: wgpu::ComputePipeline,
    flux: wgpu::ComputePipeline,
    update_water: wgpu::ComputePipeline,
    water_erosion: wgpu::ComputePipeline,
    sediment_movement: wgpu::ComputePipeline,
    thermal_erosion: wgpu::ComputePipeline,
}
impl WaterSimCompute {
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            rain: Self::create_pipeline(gpu, "rain"),
            flux: Self::create_pipeline(gpu, "flux"),
            update_water: Self::create_pipeline(gpu, "update_water"),
            water_erosion: Self::create_pipeline(gpu, "water_erosion"),
            sediment_movement: Self::create_pipeline(gpu, "sediment_movement"),
            thermal_erosion: Self::create_pipeline(gpu, "thermal_erosion"),
        }
    }
    fn create_pipeline(gpu: &Gpu, entry_point: &str) -> wgpu::ComputePipeline {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("WaterSimCompute.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(wgsl_terrain_preprocess(
                    include_file!("water_sim.wgsl"),
                ))),
            });

        gpu.device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(
                    &gpu.device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[&gpu.device.create_bind_group_layout(
                                &wgpu::BindGroupLayoutDescriptor {
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
                                },
                            )],
                            push_constant_ranges: &[],
                        }),
                ),
                module: &shader,
                entry_point,
            })
    }
    pub fn run(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        texture: &TextureView,
        size: UVec2,
        config: &WaterSimConfig,
    ) {
        let param_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Simulation Parameter Buffer"),
                contents: bytemuck::cast_slice(&[config.params]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        for pipeline in [
            &self.rain,
            &self.flux,
            &self.update_water,
            &self.water_erosion,
            &self.sediment_movement,
            &self.thermal_erosion,
        ] {
            let bind_group_layout = pipeline.get_bind_group_layout(0);
            let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: param_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(size.x, size.y, 1);
        }
    }
}
