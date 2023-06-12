use std::borrow::Cow;

use ambient_gpu::{gpu::Gpu, texture::TextureView};
use ambient_std::include_file;
use glam::UVec2;

use crate::wgsl_terrain_preprocess;

#[derive(Debug)]
pub struct NormalmapFromHeightmapCompute {
    pipeline: wgpu::ComputePipeline,
}
impl NormalmapFromHeightmapCompute {
    pub fn new(gpu: &Gpu) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("NormalmapFromHeightmapCompute.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(
                    [wgsl_terrain_preprocess(include_file!("normalmap.wgsl"))].join("\n"),
                )),
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
                                                access: wgpu::StorageTextureAccess::ReadOnly,
                                                format: wgpu::TextureFormat::R32Float,
                                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                            },
                                            count: None,
                                        },
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 1,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::StorageTexture {
                                                access: wgpu::StorageTextureAccess::WriteOnly,
                                                format: wgpu::TextureFormat::Rgba32Float,
                                                view_dimension: wgpu::TextureViewDimension::D2,
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
        heightmap: &TextureView,
        normalmap: &TextureView,
        size: UVec2,
    ) {
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(heightmap),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(normalmap),
                },
            ],
        });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(size.x, size.y, 1);
    }
}
