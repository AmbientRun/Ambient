use std::sync::Arc;

use ambient_core::gpu_ecs::{GpuWorldShaderModuleKey, ENTITIES_BIND_GROUP};
use ambient_ecs::{EntityId, World};
use ambient_gpu::{
    gpu::Gpu,
    multi_buffer::TypedMultiBuffer,
    shader_module::{BindGroupDesc, ComputePipeline, Shader, ShaderIdent, ShaderModule},
    typed_buffer::TypedBuffer,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    include_file,
};
use glam::{uvec2, UVec2, UVec3};
use parking_lot::Mutex;
use wgpu::{
    BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType,
    ShaderStages,
};

use crate::{
    get_mesh_meta_module, DrawIndexedIndirect, MaterialLayout, PostSubmitFunc, GLOBALS_BIND_GROUP,
};

use super::{get_defs_module, PrimitiveIndex};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CollectPrimitive {
    pub entity_loc: UVec2,
    pub primitive_id: u32,
    pub material_index: u32,
}

impl CollectPrimitive {
    pub fn from_primitive(
        world: &World,
        id: EntityId,
        primitive_index: PrimitiveIndex,
        material_index: u32,
    ) -> Self {
        let loc = world.entity_loc(id).unwrap();
        Self {
            entity_loc: uvec2(loc.archetype as u32, loc.index as u32),
            primitive_id: primitive_index as u32,
            material_index,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct DrawCountState {
    counts: Vec<u32>,
    /// The last tick that the counts were updated. Used for enforcing ordering
    last_tick: u64,
}

impl DrawCountState {
    pub fn update(&mut self, counts: Vec<u32>, tick: u64) {
        if self.last_tick > tick {
            tracing::warn!("Skipping count update because it's out of date");
            return;
        }

        tracing::debug!(?counts, "Updating counts");

        self.last_tick = tick;
        self.counts = counts;
    }

    #[inline]
    pub fn counts(&self) -> &[u32] {
        self.counts.as_ref()
    }

    pub fn counts_mut(&mut self) -> &mut Vec<u32> {
        &mut self.counts
    }
}

/// Contains the primitive input and indirect output buffers for GPU driven rendering
pub(crate) struct RendererCollectState {
    pub params: TypedBuffer<RendererCollectParams>,
    pub commands: TypedBuffer<DrawIndexedIndirect>,
    pub counts: TypedBuffer<u32>,
    #[cfg(any(target_os = "macos", target_os = "unknown"))]
    /// Multi draw indexed indirect is not supported on macOS
    pub(crate) counts_cpu: Arc<Mutex<DrawCountState>>,
    pub tick: u64,
    pub material_layouts: TypedBuffer<MaterialLayout>,
}

impl RendererCollectState {
    pub fn new(gpu: &Gpu) -> Self {
        log::debug!("Setting up renderer collect state");
        Self {
            params: TypedBuffer::new_init(
                gpu,
                "RendererCollectState.params",
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                &[RendererCollectParams::default()],
            ),
            commands: TypedBuffer::new(
                gpu,
                "RendererCollectState.commands",
                1,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::INDIRECT,
            ),
            counts: TypedBuffer::new(
                gpu,
                "RendererCollectState.counts",
                1,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::INDIRECT,
            ),
            #[cfg(any(target_os = "macos", target_os = "unknown"))]
            counts_cpu: Arc::new(Default::default()),
            material_layouts: TypedBuffer::new(
                gpu,
                "RendererCollectState.materials",
                1,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::INDIRECT,
            ),
            tick: 0,
        }
    }
    pub fn set_camera(&self, gpu: &Gpu, camera: u32) {
        let collect_params = RendererCollectParams {
            camera,
            _padding: Default::default(),
        };
        self.params.write(gpu, 0, &[collect_params]);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RendererCollectParams {
    pub camera: u32,
    pub _padding: UVec3,
}

const COLLECT_WORKGROUP_SIZE: u32 = 32;
// const COLLECT_CHUNK_SIZE: u32 = 256;

/// This collects primitives into indirect draw buffers
#[allow(dead_code)]
pub struct RendererCollect {
    pipeline: ComputePipeline,
    layout: Arc<BindGroupLayout>,
}

impl RendererCollect {
    pub fn new(gpu: &Gpu, assets: &AssetCache) -> Self {
        let layout_desc = BindGroupDesc {
            label: "RendererCollect.layout".into(),
            entries: vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
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
        let shader = Shader::new(
            assets,
            "collect",
            &[
                GLOBALS_BIND_GROUP,
                ENTITIES_BIND_GROUP,
                "RendererCollect.layout",
            ],
            &ShaderModule::new("RendererCollect", include_file!("collect.wgsl"))
                .with_ident(ShaderIdent::constant(
                    "COLLECT_WORKGROUP_SIZE",
                    COLLECT_WORKGROUP_SIZE,
                ))
                // .with_ident(ShaderIdent::constant(
                //     "COLLECT_CHUNK_SIZE",
                //     COLLECT_CHUNK_SIZE,
                // ))
                .with_binding_desc(layout_desc)
                .with_dependency(get_defs_module())
                .with_dependency(get_mesh_meta_module(0))
                .with_dependency(GpuWorldShaderModuleKey { read_only: true }.get(assets)),
        )
        .unwrap();

        let pipeline = shader.to_compute_pipeline(gpu, "main");

        Self { pipeline, layout }
    }

    /// Updates the GPU side data needed for indirect rendering.
    ///
    /// **Note**:
    pub(crate) fn update(
        &self,
        gpu: &Gpu,
        material_layouts: &[MaterialLayout],
        collect_state: &mut RendererCollectState,
    ) {
        tracing::info!(
            from = collect_state.counts.len(),
            to = material_layouts.len(),
            "Resizing counts buffer",
        );
        // if collect_state.counts.len() != material_layouts.len() {
        let counts = vec![0; material_layouts.len()];

        collect_state.counts.fill(gpu, &counts, |_| {});
        // }

        // tracing::debug!("material_layouts: {material_layouts:?}");

        collect_state
            .material_layouts
            .fill(gpu, material_layouts, |_| {});
        collect_state.tick += 1;
    }

    /// Computes indirect draw commands using culling
    #[ambient_profiling::function]
    pub(crate) fn compute_indirect(
        &self,
        gpu: &Gpu,
        assets: &AssetCache,
        encoder: &mut wgpu::CommandEncoder,
        _post_submit: &mut Vec<PostSubmitFunc>,
        mesh_meta_bind_group: &wgpu::BindGroup,
        entities_bind_group: &wgpu::BindGroup,
        input_primitives: &TypedMultiBuffer<CollectPrimitive>,
        output: &mut RendererCollectState,
        primitives_count: u32,
    ) {
        output.commands.set_len(gpu, primitives_count as usize);

        // Avoid binding 0 size buffers
        if primitives_count == 0 {
            return;
        }

        assert_eq!(
            input_primitives.total_len(),
            primitives_count as u64,
            "Expected count {primitives_count}",
        );

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: output.params.as_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: input_primitives.buffer().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: output.commands.as_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: output.counts.as_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: output.material_layouts.as_binding(),
                },
            ],
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Collect"),
            });

            cpass.set_pipeline(self.pipeline.pipeline());

            for (i, bind_group) in [mesh_meta_bind_group, entities_bind_group, &bind_group]
                .iter()
                .enumerate()
            {
                cpass.set_bind_group(i as _, bind_group, &[]);
            }

            // Divide up all the primitives among `x` workgroups
            let x = (primitives_count as f32 / COLLECT_WORKGROUP_SIZE as f32).ceil() as u32;
            tracing::debug!("Dispatching {x} workgroups");

            cpass.dispatch_workgroups(x, 1, 1);
        }

        #[cfg(any(target_os = "macos", target_os = "unknown"))]
        {
            use ambient_core::RuntimeKey;

            let buffs = CollectCountStagingBuffersKey.get(assets);
            let staging = buffs.take_buffer(gpu, output.counts.len());

            encoder.copy_buffer_to_buffer(
                output.counts.buffer(),
                0,
                staging.buffer(),
                0,
                output.counts.byte_len(),
            );

            let counts_res = output.counts_cpu.clone();
            let runtime = RuntimeKey.get(assets);
            let post_submit_gpu = ambient_gpu::gpu::GpuKey.get(assets);
            let tick = output.tick;

            _post_submit.push(Box::new(move || {
                runtime.spawn(async move {
                    if let Ok(res) = staging.read(&post_submit_gpu, ..).await {
                        let len = res.len();

                        {
                            let mut count_state = counts_res.lock();
                            assert_ne!(count_state.last_tick, tick);

                            count_state.update(res, tick);
                        }
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
    fn load(&self, _assets: AssetCache) -> CollectCountStagingBuffers {
        CollectCountStagingBuffers::new()
    }
}

#[derive(Clone)]
#[allow(dead_code)]
struct CollectCountStagingBuffers {
    buffers: Arc<Mutex<Vec<TypedBuffer<u32>>>>,
}
impl CollectCountStagingBuffers {
    fn new() -> Self {
        Self {
            buffers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[cfg(any(target_os = "macos", target_os = "unknown"))]
    fn take_buffer(&self, gpu: &Gpu, size: usize) -> TypedBuffer<u32> {
        match self.buffers.lock().pop() {
            Some(mut buffer) => {
                buffer.set_len_discard(gpu, size);
                buffer
            }
            None => TypedBuffer::<u32>::new(
                gpu,
                "RendererCollectState.counts_staging",
                size,
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
        }
    }

    #[cfg(any(target_os = "macos", target_os = "unknown"))]
    fn return_buffer(&self, buffer: TypedBuffer<u32>) {
        self.buffers.lock().push(buffer)
    }
}
