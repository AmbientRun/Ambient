use std::{collections::HashMap, sync::Arc};

use ambient_ecs::{ArchetypeFilter, World};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    shader_module::{BindGroupDesc, ComputePipeline, Shader, ShaderIdent, ShaderModule},
    typed_buffer::TypedBuffer,
};
use ambient_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use glam::{uvec4, UVec4};
use wgpu::BindGroupLayoutEntry;

use super::{gpu_world, GpuWorldShaderModuleKey, ENTITIES_BIND_GROUP};

const GPU_WORLD_UPDATE_CHUNK_SIZE: u32 = 256;
const GPU_ECS_UPDATE_BIND_GROUP: &str = "GPU_ECS_UPDATE_BIND_GROUP";
const GPU_ECS_WORKGROUP_SIZE: u32 = 32;

pub struct GpuWorldUpdater {
    pub gpu: Arc<Gpu>,
    pub pipeline: ComputePipeline,
    filter: ArchetypeFilter,
    chunks: TypedBuffer<UVec4>,
}
impl GpuWorldUpdater {
    pub fn new(
        assets: AssetCache,
        label: String,
        filter: ArchetypeFilter,
        mut modules: Vec<Arc<ShaderModule>>,
        body: impl Into<String>,
    ) -> Self {
        modules.insert(0, GpuWorldShaderModuleKey { read_only: false }.get(&assets));

        let module = ShaderModule::new("GpuWorldUpdate", include_str!("update.wgsl"))
            .with_binding(
                GPU_ECS_UPDATE_BIND_GROUP,
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            )
            .with_ident(ShaderIdent::raw("UPDATE_BODY", body.into()))
            .with_ident(ShaderIdent::constant("GPU_WORLD_UPDATE_CHUNK_SIZE", GPU_WORLD_UPDATE_CHUNK_SIZE))
            .with_ident(ShaderIdent::constant("GPU_ECS_WORKGROUP_SIZE", GPU_ECS_WORKGROUP_SIZE))
            .with_dependencies(modules);

        let gpu = GpuKey.get(&assets);

        let shader = Shader::from_modules(&assets, format!("GpuWorldUpdate.{label}"), &module);
        let pipeline = shader.to_compute_pipeline(&gpu, "main");

        Self {
            pipeline,
            filter,
            chunks: TypedBuffer::new(
                gpu.clone(),
                "GpuWorldUpdate.chunks",
                1,
                1,
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            ),
            gpu,
        }
    }

    pub fn run(&mut self, world: &World, binding_context: &[(&str, &wgpu::BindGroup)]) {
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.run_with_encoder(&mut encoder, world, binding_context);
        self.gpu.queue.submit(Some(encoder.finish()));
    }

    pub fn run_with_encoder(&mut self, encoder: &mut wgpu::CommandEncoder, world: &World, binding_context: &[(&str, &wgpu::BindGroup)]) {
        let mut chunks = Vec::new();
        for arch in self.filter.iter_archetypes(world) {
            if arch.entity_count() == 0 {
                continue;
            }
            let chunk_count = ((arch.entity_count() as f32 / GPU_WORLD_UPDATE_CHUNK_SIZE as f32).ceil() as usize).max(1);
            for i in 0..chunk_count {
                chunks.push(uvec4(arch.id as u32, i as u32 * GPU_WORLD_UPDATE_CHUNK_SIZE, arch.entity_count() as u32, 0));
            }
        }
        if chunks.is_empty() {
            return;
        }

        self.chunks.fill(&chunks, |_| {});

        let bind_group = self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: self.pipeline.shader().get_bind_group_layout_by_name(GPU_ECS_UPDATE_BIND_GROUP).unwrap(),
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: self.chunks.buffer().as_entire_binding() }],
        });

        let gpu_world = world.resource(gpu_world());
        let gpu_world_bind_group = {
            let gpu_world = gpu_world.lock();
            gpu_world.create_bind_group(false)
        };

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("GpuWorldUpdate") });
        cpass.set_pipeline(self.pipeline.pipeline());

        for (name, bind_group) in
            binding_context.iter().chain([&(ENTITIES_BIND_GROUP, &gpu_world_bind_group), &(GPU_ECS_UPDATE_BIND_GROUP, &bind_group)])
        {
            let index = self.pipeline.get_bind_group_index_by_name(name).unwrap();

            cpass.set_bind_group(index, bind_group, &[]);
        }

        debug_assert_eq!(GPU_WORLD_UPDATE_CHUNK_SIZE % GPU_ECS_WORKGROUP_SIZE, 0);
        cpass.dispatch_workgroups(GPU_WORLD_UPDATE_CHUNK_SIZE / GPU_ECS_WORKGROUP_SIZE, chunks.len() as u32, 1);
    }
}
