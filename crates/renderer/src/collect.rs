use std::sync::Arc;

use ambient_core::gpu_ecs::{GpuWorldShaderModuleKey, ENTITIES_BIND_GROUP};
use ambient_ecs::{EntityId, World};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    mesh_buffer::get_mesh_buffer_types,
    multi_buffer::TypedMultiBuffer,
    shader_module::{BindGroupDesc, ComputePipeline, Shader, ShaderModule, ShaderModuleIdentifier},
    typed_buffer::TypedBuffer,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    include_file,
};
use glam::{uvec2, UVec2, UVec3};
use parking_lot::Mutex;
use wgpu::{BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages};

use super::{get_defs_module, get_resources_module, DrawIndexedIndirect, PrimitiveIndex, RESOURCES_BIND_GROUP};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CollectPrimitive {
    entity_loc: UVec2,
    primitive_index: u32,
    material_index: u32,
}
impl CollectPrimitive {
    pub fn from_primitive(world: &World, id: EntityId, primitive_index: PrimitiveIndex, material_index: u32) -> Self {
        let loc = world.entity_loc(id).unwrap();
        Self { entity_loc: uvec2(loc.archetype as u32, loc.index as u32), primitive_index: primitive_index as u32, material_index }
    }
}

pub struct RendererCollectState {
    pub params: TypedBuffer<RendererCollectParams>,
    pub commands: TypedBuffer<DrawIndexedIndirect>,
    pub counts: TypedBuffer<u32>,
    #[cfg(target_os = "macos")]
    pub counts_cpu: Arc<Mutex<Vec<u32>>>,
    pub material_layouts: TypedBuffer<UVec2>,
}
impl RendererCollectState {
    pub fn new(assets: &AssetCache) -> Self {
        log::info!("Setting up renderer collect state");
        let gpu = GpuKey.get(assets);
        Self {
            params: TypedBuffer::new(
                gpu.clone(),
                "RendererCollectState.params",
                1,
                1,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            commands: TypedBuffer::new(
                gpu.clone(),
                "RendererCollectState.commands",
                1,
                1,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDIRECT,
            ),
            counts: TypedBuffer::new(
                gpu.clone(),
                "RendererCollectState.counts",
                1,
                1,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDIRECT,
            ),
            #[cfg(target_os = "macos")]
            counts_cpu: Arc::new(Mutex::new(Vec::new())),
            material_layouts: TypedBuffer::new(
                gpu,
                "RendererCollectState.materials",
                1,
                1,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDIRECT,
            ),
        }
    }
    pub fn set_camera(&self, camera: u32) {
        let collect_params = RendererCollectParams { camera, _padding: Default::default() };
        self.params.write(0, &[collect_params]);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RendererCollectParams {
    pub camera: u32,
    pub _padding: UVec3,
}

const COLLECT_WORKGROUP_SIZE: u32 = 32;
const COLLECT_CHUNK_SIZE: u32 = 256;

/// This collects primitives into indirect draw buffers
#[allow(dead_code)]
pub struct RendererCollect {
    gpu: Arc<Gpu>,
    pipeline: ComputePipeline,
    layout: Arc<BindGroupLayout>,
    assets: AssetCache,
}

impl RendererCollect {
    pub fn new(assets: &AssetCache) -> Self {
        let gpu = GpuKey.get(assets);

        let layout_desc = BindGroupDesc {
            label: "RendererCollect.layout".into(),
            entries: vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        };

        let layout = layout_desc.load(assets.clone());
        let shader = Shader::from_modules(
            assets,
            "collect",
            &[
                get_defs_module(assets),
                get_mesh_buffer_types(),
                get_resources_module(),
                GpuWorldShaderModuleKey { read_only: true }.get(assets),
                ShaderModule::new(
                    "RendererCollect",
                    include_file!("collect.wgsl"),
                    vec![
                        layout_desc.into(),
                        ShaderModuleIdentifier::constant("COLLECT_WORKGROUP_SIZE", COLLECT_WORKGROUP_SIZE),
                        ShaderModuleIdentifier::constant("COLLECT_CHUNK_SIZE", COLLECT_CHUNK_SIZE),
                    ],
                ),
            ],
        );

        let pipeline = shader.to_compute_pipeline(&gpu, "main");
        Self { gpu, pipeline, layout, assets: assets.clone() }
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::ptr_arg)]
    #[profiling::function]
    pub fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        _post_submit: &mut Vec<Box<dyn FnOnce() + Send + Send>>,
        resources_bind_group: &wgpu::BindGroup,
        entities_bind_group: &wgpu::BindGroup,
        input_primitives: &TypedMultiBuffer<CollectPrimitive>,
        output: &mut RendererCollectState,
        primitives_count: u32,
        material_layouts: Vec<UVec2>,
    ) {
        if primitives_count == 0 {
            return;
        }
        output.commands.resize(primitives_count as u64, true);
        let counts = vec![0; material_layouts.len()];
        output.counts.fill(&counts, |_| {});
        output.material_layouts.fill(&material_layouts, |_| {});

        let bind_group = self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: output.params.buffer().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: input_primitives.buffer().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: output.commands.buffer().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: output.counts.buffer().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: output.material_layouts.buffer().as_entire_binding() },
            ],
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Collect") });
            cpass.set_pipeline(self.pipeline.pipeline());

            for (name, group) in [(RESOURCES_BIND_GROUP, resources_bind_group), (ENTITIES_BIND_GROUP, entities_bind_group)] {
                let id = self.pipeline.get_bind_group_index_by_name(name).unwrap();
                cpass.set_bind_group(id, group, &[]);
            }
            cpass.set_bind_group(2, &bind_group, &[]);
            let count = (primitives_count as f32 / COLLECT_WORKGROUP_SIZE as f32).ceil() as u32;
            let width = if count < COLLECT_CHUNK_SIZE { count } else { COLLECT_CHUNK_SIZE };
            let height = (count as f32 / COLLECT_CHUNK_SIZE as f32).ceil() as u32;
            cpass.dispatch_workgroups(width, height, 1);
        }

        #[cfg(target_os = "macos")]
        {
            use ambient_core::RuntimeKey;

            let buffs = CollectCountStagingBuffersKey.get(&self.assets);
            let staging = buffs.take_buffer(output.counts.len());
            encoder.copy_buffer_to_buffer(output.counts.buffer(), 0, staging.buffer(), 0, output.counts.size());
            let counts_res = output.counts_cpu.clone();
            let runtime = RuntimeKey.get(&self.assets);
            _post_submit.push(Box::new(move || {
                runtime.spawn(async move {
                    if let Ok(res) = staging.read(.., false).await {
                        *counts_res.lock() = res;
                        buffs.return_buffer(staging);
                    }
                });
            }))
        }
    }
}

#[derive(Clone, Debug)]
struct CollectCountStagingBuffersKey;
impl SyncAssetKey<CollectCountStagingBuffers> for CollectCountStagingBuffersKey {
    fn load(&self, assets: AssetCache) -> CollectCountStagingBuffers {
        CollectCountStagingBuffers::new(GpuKey.get(&assets))
    }
}

#[derive(Clone)]
#[allow(dead_code)]
struct CollectCountStagingBuffers {
    gpu: Arc<Gpu>,
    buffers: Arc<Mutex<Vec<TypedBuffer<u32>>>>,
}
impl CollectCountStagingBuffers {
    fn new(gpu: Arc<Gpu>) -> Self {
        Self { gpu, buffers: Arc::new(Mutex::new(Vec::new())) }
    }

    #[cfg(target_os = "macos")]
    fn take_buffer(&self, size: u64) -> TypedBuffer<u32> {
        match self.buffers.lock().pop() {
            Some(mut buffer) => {
                buffer.resize(size, false);
                buffer
            }
            None => TypedBuffer::<u32>::new(
                self.gpu.clone(),
                "RendererCollectState.counts_staging",
                size,
                size,
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
        }
    }

    #[cfg(target_os = "macos")]
    fn return_buffer(&self, buffer: TypedBuffer<u32>) {
        self.buffers.lock().push(buffer)
    }
}
