// Ported from https://github.com/SebLague/Hydraulic-Erosion/blob/master/Assets/Scripts/Erosion.cs (MIT)

use std::{borrow::Cow, f32::consts::PI};

use ambient_editor_derive::ElementEditor;
use ambient_gpu::{gpu::Gpu, texture::TextureView};
use ambient_native_std::include_file;
use glam::{ivec2, vec2, IVec2, IVec3, UVec2, Vec2};
use itertools::Itertools;
use rand::prelude::*;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use crate::wgsl_terrain_preprocess;

#[derive(Clone, Debug, Serialize, Deserialize, ElementEditor)]
pub struct HydraulicErosionConfig {
    pub params: HydraulicErosionParams,
    #[editor(slider, min = 1, max = 8)]
    pub drop_radius: i32,
    #[editor(slider, min = 0., max = 4., logarithmic)]
    pub drops_per_m2: f32,
    pub seed: u64,
    pub brush_position: Vec2,
    pub brush_radius: f32,
}
impl Default for HydraulicErosionConfig {
    fn default() -> Self {
        Self {
            params: Default::default(),
            drop_radius: 3,
            drops_per_m2: 0.01,
            seed: 0,
            brush_position: Vec2::ZERO,
            brush_radius: 100.,
        }
    }
}

#[repr(C)]
#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, ElementEditor, bytemuck::Pod, bytemuck::Zeroable,
)]
pub struct HydraulicErosionParams {
    #[editor(hidden)]
    pub heightmap_size: IVec2,
    pub border_size: i32,

    #[editor(slider, min = 0, max = 200)]
    pub max_lifetime: i32,
    #[editor(slider, logarithmic)]
    pub inertia: f32,
    #[editor(slider, min = 0., max = 64.)]
    pub capacity: f32,
    #[editor(slider, min = 0., max = 0.05, logarithmic)]
    pub min_slope: f32,
    #[editor(slider, logarithmic)]
    pub deposition: f32,
    #[editor(slider, logarithmic)]
    pub erosion: f32,
    #[editor(slider, max = 0.5, logarithmic)]
    pub evaporation: f32,

    #[editor(slider, min = 0., max = 10.)]
    pub gravity: f32,
    #[editor(slider, min = 0., max = 10.)]
    pub start_velocity: f32,
    #[editor(slider, min = 0., max = 10.)]
    pub start_water: f32,

    #[editor(hidden)]
    pub _padding: IVec3,
}
impl Default for HydraulicErosionParams {
    fn default() -> Self {
        Self {
            heightmap_size: IVec2::ZERO,
            border_size: 5,
            max_lifetime: 120,
            inertia: 0.01,
            capacity: 7.,
            min_slope: 0.001,
            deposition: 0.05,
            erosion: 0.003,
            evaporation: 0.03,
            gravity: 0.1,
            start_velocity: 10.,
            start_water: 7.,
            _padding: Default::default(),
        }
    }
}

pub struct HydraulicErosionCompute {
    pipeline: wgpu::ComputePipeline,
}
impl HydraulicErosionCompute {
    pub fn new(gpu: &Gpu) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("HydraulicErosionCompute.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(wgsl_terrain_preprocess(
                    include_file!("hydraulic_erosion.wgsl"),
                ))),
            });

        let pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: Some(&gpu.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
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
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 2,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Buffer {
                                                ty: wgpu::BufferBindingType::Storage {
                                                    read_only: true,
                                                },
                                                has_dynamic_offset: false,
                                                min_binding_size: None,
                                            },
                                            count: None,
                                        },
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 3,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Buffer {
                                                ty: wgpu::BufferBindingType::Storage {
                                                    read_only: true,
                                                },
                                                has_dynamic_offset: false,
                                                min_binding_size: None,
                                            },
                                            count: None,
                                        },
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 4,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Buffer {
                                                ty: wgpu::BufferBindingType::Storage {
                                                    read_only: true,
                                                },
                                                has_dynamic_offset: false,
                                                min_binding_size: None,
                                            },
                                            count: None,
                                        },
                                    ],
                                },
                            )],
                            push_constant_ranges: &[],
                        },
                    )),
                    module: &shader,
                    entry_point: "main",
                });
        Self { pipeline }
    }
    pub fn run(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        texture: &TextureView,
        size: UVec2,
        config: &HydraulicErosionConfig,
    ) {
        let drops = (((size.x * size.y) as f32 * config.drops_per_m2) as usize).max(1);

        let param_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Simulation Parameter Buffer"),
                contents: bytemuck::cast_slice(&[config.params]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // Create brush
        let mut brush_positions = Vec::new();
        let mut brush_weights = Vec::new();

        let mut weight_sum = 0.;
        for brush_y in (-config.drop_radius)..(config.drop_radius + 1) {
            for brush_x in (-config.drop_radius)..(config.drop_radius + 1) {
                let sqr_dst = brush_x * brush_x + brush_y * brush_y;
                if sqr_dst < config.drop_radius * config.drop_radius {
                    brush_positions.push(ivec2(brush_x, brush_y));
                    let brush_weight = 1. - (sqr_dst as f32).sqrt() / config.drop_radius as f32;
                    weight_sum += brush_weight;
                    brush_weights.push(brush_weight);
                }
            }
        }
        for brush_weight in &mut brush_weights {
            *brush_weight /= weight_sum;
        }
        let mut rng = Pcg64::seed_from_u64(config.seed);
        let random_positions = (0..drops)
            .map(|_| {
                let r = config.brush_radius * rng.gen::<f32>().sqrt();
                let theta = rng.gen::<f32>() * 2. * PI;
                (vec2(theta.cos(), theta.sin()) * r + config.brush_position).as_ivec2()
                // ivec2(rng.gen_range(0..(size.x as i32 - 1)), rng.gen_range(0..(size.y as i32 - 1)))
            })
            .collect_vec();

        let brush_positions = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Simulation Parameter Buffer"),
                contents: bytemuck::cast_slice(&brush_positions),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let brush_weights = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Simulation Parameter Buffer"),
                contents: bytemuck::cast_slice(&brush_weights),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let random_positions = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Simulation Parameter Buffer"),
                contents: bytemuck::cast_slice(&random_positions),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: random_positions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: brush_positions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: brush_weights.as_entire_binding(),
                },
            ],
        });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        let max_dim = 65535;
        let count = (drops as f32 / 32.).ceil() as u32;
        for i in 0..(count as f32 / max_dim as f32).ceil() as u32 {
            let c = if i == 0 { count % max_dim } else { max_dim };
            cpass.dispatch_workgroups(c, 1, 1);
        }
    }
}
