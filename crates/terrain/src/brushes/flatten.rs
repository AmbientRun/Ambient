use std::borrow::Cow;

use glam::{UVec2, Vec2};
use kiwi_gpu::{gpu::Gpu, texture::TextureView, wgsl_utils::wgsl_interpolate};
use kiwi_std::include_file;
use wgpu::{
    util::DeviceExt, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages, TextureFormat,
    TextureViewDimension,
};

use super::BrushWGSL;
use crate::{wgsl_terrain_preprocess, TERRAIN_LAYERS};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlattenBrushParams {
    pub brush: BrushWGSL,
    pub start_texel: UVec2,
    pub heightmap_world_position: Vec2,
    pub heightmap_world_texel_size: f32,

    pub _padding: UVec2,
}
impl Default for FlattenBrushParams {
    fn default() -> Self {
        Self {
            brush: Default::default(),
            heightmap_world_position: Vec2::ZERO,
            heightmap_world_texel_size: 0.,
            start_texel: Default::default(),

            _padding: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct FlattenBrush {
    pipeline: wgpu::ComputePipeline,
}
impl FlattenBrush {
    pub fn new(gpu: &Gpu) -> Self {
        let shader =
            [&wgsl_interpolate() as &str, &include_file!("brush.wgsl"), &wgsl_terrain_preprocess(include_file!("flatten.wgsl"))].join("\n");
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("FlattenBrush.shader"),
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
                            ty: BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::ReadOnly,
                                format: TextureFormat::R32Float,
                                view_dimension: TextureViewDimension::D2Array,
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
    pub fn run(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        heightmap: &TextureView,
        start_heightmap: &TextureView,
        size: UVec2,
        params: &FlattenBrushParams,
    ) {
        let param_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Flatten Parameter Buffer"),
            contents: bytemuck::bytes_of(params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(heightmap) },
                wgpu::BindGroupEntry { binding: 1, resource: param_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(start_heightmap) },
            ],
        });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(size.x, size.y, TERRAIN_LAYERS);
    }
}
