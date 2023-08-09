use std::{borrow::Cow, sync::Arc};

use ambient_native_std::asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt};
use glam::Vec4;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages,
    TextureViewDimension,
};

use super::{
    gpu::{Gpu, GpuKey},
    texture_format_to_wgsl_storage_format,
};

#[derive(Debug, Clone)]
pub struct FillerKey {
    pub format: wgpu::TextureFormat,
}
impl SyncAssetKey<Arc<Filler>> for FillerKey {
    fn load(&self, assets: AssetCache) -> Arc<Filler> {
        let gpu = GpuKey.get(&assets);
        Arc::new(Filler::new(&gpu, self.format))
    }
}

pub struct Filler {
    pipeline: wgpu::ComputePipeline,
}
impl Filler {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let shader = format!(
            "

@group(0)
@binding(0)
var output: texture_storage_2d<{}, write>;

struct Params {{
    color: vec4<f32>,
}};
@group(0)
@binding(1)
var<uniform> params: Params;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    textureStore(output, vec2<i32>(global_id.xy), params.color);
}}

        ",
            texture_format_to_wgsl_storage_format(format)
        );
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Filler.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader)),
            });

        let pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Filler"),
                    layout: Some(&gpu.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
                            label: Some("Filler"),
                            bind_group_layouts: &[&gpu.device.create_bind_group_layout(
                                &BindGroupLayoutDescriptor {
                                    label: Some("Filler"),
                                    entries: &[
                                        BindGroupLayoutEntry {
                                            binding: 0,
                                            visibility: ShaderStages::COMPUTE,
                                            ty: BindingType::StorageTexture {
                                                access: wgpu::StorageTextureAccess::WriteOnly,
                                                format,
                                                view_dimension: TextureViewDimension::D2,
                                            },
                                            count: None,
                                        },
                                        BindGroupLayoutEntry {
                                            binding: 1,
                                            visibility: ShaderStages::COMPUTE,
                                            ty: BindingType::Buffer {
                                                ty: BufferBindingType::Uniform,
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
    pub fn run(&self, gpu: &Gpu, target: &wgpu::TextureView, size: wgpu::Extent3d, color: Vec4) {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Filler.run"),
            });
        self.run_with_encoder(gpu, &mut encoder, target, size, color);
        gpu.queue.submit(Some(encoder.finish()));
    }
    pub fn run_with_encoder(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        size: wgpu::Extent3d,
        color: Vec4,
    ) {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        struct FillParams {
            pub color: Vec4,
        }

        let params = FillParams { color };
        let param_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Filler params"),
                contents: bytemuck::cast_slice(&[params]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Filler"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(target),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: param_buffer.as_entire_binding(),
                },
            ],
        });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Filler"),
        });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(size.width, size.height, size.depth_or_array_layers);
    }
}
