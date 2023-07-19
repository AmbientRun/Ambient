use std::sync::Arc;

use ambient_ecs::{ArchetypeFilter, World};
use ambient_gpu::{
    gpu::Gpu,
    shader_module::{BindGroupDesc, ComputePipeline, Shader, ShaderIdent, ShaderModule},
    typed_buffer::TypedBuffer,
};
use ambient_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use glam::{uvec4, UVec4};
use itertools::Itertools;
use wgpu::{BindGroupLayout, BindGroupLayoutEntry};

use super::{gpu_world, GpuWorldShaderModuleKey, ENTITIES_BIND_GROUP};

const GPU_WORLD_UPDATE_CHUNK_SIZE: u32 = 256;
const GPU_ECS_UPDATE_BIND_GROUP: &str = "GPU_ECS_UPDATE_BIND_GROUP";
const GPU_ECS_WORKGROUP_SIZE: u32 = 32;

fn get_gpu_update_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        label: GPU_ECS_UPDATE_BIND_GROUP.into(),

        entries: vec![BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    }
}

pub struct GpuWorldUpdater {
    pub pipeline: ComputePipeline,
    layout: Arc<BindGroupLayout>,
    filter: ArchetypeFilter,
    chunks: TypedBuffer<UVec4>,
}

impl GpuWorldUpdater {
    pub fn new(
        gpu: &Gpu,
        assets: AssetCache,
        label: String,
        filter: ArchetypeFilter,
        mut modules: Vec<Arc<ShaderModule>>,
        bind_groups: &[&str],
        body: impl Into<String>,
    ) -> Self {
        modules.insert(0, GpuWorldShaderModuleKey { read_only: false }.get(&assets));

        let module = ShaderModule::new("GpuWorldUpdate", include_str!("update.wgsl"))
            .with_binding_desc(get_gpu_update_layout())
            .with_ident(ShaderIdent::raw("UPDATE_BODY", body.into()))
            .with_ident(ShaderIdent::constant(
                "GPU_WORLD_UPDATE_CHUNK_SIZE",
                GPU_WORLD_UPDATE_CHUNK_SIZE,
            ))
            .with_ident(ShaderIdent::constant(
                "GPU_ECS_WORKGROUP_SIZE",
                GPU_ECS_WORKGROUP_SIZE,
            ))
            .with_dependencies(modules);

        let shader = Shader::new(
            &assets,
            format!("GpuWorldUpdate.{label}"),
            &[GPU_ECS_UPDATE_BIND_GROUP, ENTITIES_BIND_GROUP]
                .into_iter()
                .chain(bind_groups.iter().copied())
                .collect_vec(),
            &module,
        )
        .unwrap();
        let pipeline = shader.to_compute_pipeline(gpu, "main");
        let layout = get_gpu_update_layout().get(&assets);

        Self {
            pipeline,
            filter,
            chunks: TypedBuffer::new(
                gpu,
                "GpuWorldUpdate.chunks",
                1,
                wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::STORAGE,
            ),
            layout,
        }
    }

    pub fn run(&mut self, gpu: &Gpu, world: &World, bind_groups: &[&wgpu::BindGroup]) {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.run_with_encoder(gpu, &mut encoder, world, bind_groups);
        gpu.queue.submit(Some(encoder.finish()));
    }

    pub fn run_with_encoder(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        bind_groups: &[&wgpu::BindGroup],
    ) {
        let mut chunks = Vec::new();
        for arch in self.filter.iter_archetypes(world) {
            if arch.entity_count() == 0 {
                continue;
            }
            let chunk_count = ((arch.entity_count() as f32 / GPU_WORLD_UPDATE_CHUNK_SIZE as f32)
                .ceil() as usize)
                .max(1);
            for i in 0..chunk_count {
                chunks.push(uvec4(
                    arch.id as u32,
                    i as u32 * GPU_WORLD_UPDATE_CHUNK_SIZE,
                    arch.entity_count() as u32,
                    0,
                ));
            }
        }

        if chunks.is_empty() {
            return;
        }

        self.chunks.fill(gpu, &chunks, |_| {});

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.chunks.buffer().as_entire_binding(),
            }],
        });

        let gpu_world = world.resource(gpu_world());
        let entities_bind_group = {
            let gpu_world = gpu_world.lock();
            gpu_world.create_bind_group(gpu, false)
        };

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("GpuWorldUpdate"),
        });
        cpass.set_pipeline(self.pipeline.pipeline());

        for (i, bind_group) in [&bind_group, &entities_bind_group]
            .iter()
            .chain(bind_groups)
            .enumerate()
        {
            cpass.set_bind_group(i as _, bind_group, &[]);
        }

        debug_assert_eq!(GPU_WORLD_UPDATE_CHUNK_SIZE % GPU_ECS_WORKGROUP_SIZE, 0);
        cpass.dispatch_workgroups(
            GPU_WORLD_UPDATE_CHUNK_SIZE / GPU_ECS_WORKGROUP_SIZE,
            chunks.len() as u32,
            1,
        );
    }
}
