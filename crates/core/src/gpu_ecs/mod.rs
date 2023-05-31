use std::{collections::HashMap, sync::Arc};

use ambient_ecs::{components, ArchetypeId, Resource, System, World};
use ambient_gpu::{
    gpu::Gpu,
    shader_module::ShaderModule,
    typed_buffer::{TypedBuffer, UntypedBuffer},
};
use ambient_std::asset_cache::{AssetCache, SyncAssetKey};
use itertools::Itertools;
use parking_lot::Mutex;

mod component;
mod sync;
mod update;
use ambient_gpu::gpu::GpuKey;
use ambient_std::asset_cache::SyncAssetKeyExt;
pub use component::*;
pub use sync::*;
pub use update::*;
use wgpu::BindGroupLayout;

use crate::gpu;

components!("rendering", {
    @[Resource]
    gpu_world: Arc<Mutex<GpuWorld>>,
});

pub const ENTITIES_BIND_GROUP: &str = "ENTITIES_BIND_GROUP";

#[derive(Debug)]
pub struct GpuWorldShaderModuleKey {
    pub read_only: bool,
}

impl SyncAssetKey<Arc<ShaderModule>> for GpuWorldShaderModuleKey {
    fn load(&self, assets: AssetCache) -> Arc<ShaderModule> {
        let config = GpuWorldConfigKey.get(&assets);
        let source = config.wgsl(!self.read_only);

        Arc::new(ShaderModule::new("GpuWorld", source).with_bindings(
            (config.layout_entries(self.read_only)).map(|v| (ENTITIES_BIND_GROUP.into(), v)),
        ))
    }
}

#[derive(Debug)]
pub struct GpuWorldBindingGroupLayoutKey {
    pub read_only: bool,
}

impl SyncAssetKey<Arc<BindGroupLayout>> for GpuWorldBindingGroupLayoutKey {
    fn load(&self, assets: AssetCache) -> Arc<BindGroupLayout> {
        let config = GpuWorldConfigKey.get(&assets);
        let gpu = GpuKey.get(&assets);

        Arc::new(
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("GpuWorld"),
                    entries: &config.layout_entries(self.read_only).collect_vec(),
                }),
        )
    }
}

pub struct GpuWorld {
    layout_buffer: TypedBuffer<i32>,
    buffers: Vec<GpuComponentsBuffer>,
    assets: AssetCache,
}
impl GpuWorld {
    pub fn new_arced(gpu: &Gpu, assets: AssetCache) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(gpu, assets)))
    }
    pub fn new(gpu: &Gpu, assets: AssetCache) -> Self {
        let config = GpuWorldConfigKey.get(&assets);

        tracing::debug!(
            "Creating Gpu Ecs with buffers: {:#?}",
            config.buffers.iter().map(|v| v.format).collect_vec()
        );

        Self {
            layout_buffer: TypedBuffer::new(
                gpu,
                "GpuWorld.layout_buffer",
                1,
                1,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            ),
            buffers: config
                .buffers
                .iter()
                .map(|buffer| GpuComponentsBuffer::new(gpu, buffer.clone()))
                .collect(),
            assets,
        }
    }
    pub fn update(&mut self, gpu: &Gpu, world: &World) {
        self.layout_buffer
            .write(gpu, 0, &[world.archetypes().len() as i32]);
        for buf in self.buffers.iter_mut() {
            let layout_offset = buf.config.layout_offset(world.archetypes().len());
            buf.update(gpu, world, &mut self.layout_buffer, layout_offset as u64);
        }
    }
    pub fn create_bind_group(&self, gpu: &Gpu, read_only: bool) -> wgpu::BindGroup {
        let layout = GpuWorldBindingGroupLayoutKey { read_only }.get(&self.assets);

        let mut buffers = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: self.layout_buffer.buffer().as_entire_binding(),
        }];

        for (i, buf) in self.buffers.iter().enumerate() {
            buffers.push(wgpu::BindGroupEntry {
                binding: i as u32 + 1,
                resource: buf.data.buffer.as_entire_binding(),
            });
        }

        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &buffers,
            label: Some("EntityBuffers.bind_group"),
        })
    }

    pub fn get_buffer(
        &self,
        format: GpuComponentFormat,
        component: &str,
        archetype: ArchetypeId,
    ) -> Option<(&wgpu::Buffer, u64, u64)> {
        let buff = self
            .buffers
            .iter()
            .find(|buff| buff.config.format == format)?;
        let comp = buff.layout.get(component)?;
        let offset = *comp.get(&archetype)? as u64;
        Some((
            &buff.data.buffer,
            offset * buff.data.item_size(),
            buff.layout_version,
        ))
    }
}

pub struct GpuComponentsBuffer {
    config: GpuComponentsConfig,
    pub data: UntypedBuffer,
    pub layout: HashMap<String, HashMap<ArchetypeId, u32>>,
    layout_offsets: Vec<i32>,
    layout_buffer_offset: u64,
    layout_version: u64,
}

impl GpuComponentsBuffer {
    pub fn new(gpu: &Gpu, config: GpuComponentsConfig) -> Self {
        Self {
            data: UntypedBuffer::new(
                gpu,
                &format!("EntityBuffers.{}.data", config.format),
                1,
                1,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                config.format.size(),
            ),
            layout: config
                .components
                .iter()
                .map(|comp| (comp.name.clone(), HashMap::new()))
                .collect(),
            config,
            layout_offsets: Vec::new(),
            layout_buffer_offset: 0,
            layout_version: 0,
        }
    }
    pub fn update(
        &mut self,
        gpu: &Gpu,
        world: &World,
        layout_buffer: &mut TypedBuffer<i32>,
        layout_buffer_offset: u64,
    ) {
        let mut gpu_layout = Vec::new();
        let mut offset = 0;
        for component in &self.config.components {
            let mut layout = HashMap::new();
            for arch in world.archetypes() {
                if component.exists_for.matches_archetype(arch) {
                    layout.insert(arch.id, offset as u32);
                    let buf_len = arch.entity_count();
                    gpu_layout.push(offset);
                    let buf_size_pow2 = 2i32.pow((buf_len as f32).log2().ceil() as u32);
                    offset += buf_size_pow2;
                } else {
                    gpu_layout.push(-1);
                }
            }
            self.layout.insert(component.name.clone(), layout);
        }
        if gpu_layout != self.layout_offsets
            || offset != self.data.len() as i32
            || layout_buffer_offset != self.layout_buffer_offset
        {
            self.layout_version += 1;
        }
        self.layout_buffer_offset = layout_buffer_offset;
        self.data.resize(gpu, offset as u64, true);
        layout_buffer.resize(gpu, layout_buffer_offset + gpu_layout.len() as u64, true);
        layout_buffer.write(gpu, layout_buffer_offset, &gpu_layout);
        self.layout_offsets = gpu_layout;
    }
}

#[derive(Debug)]
pub struct GpuWorldUpdate;
impl System<GpuWorldSyncEvent> for GpuWorldUpdate {
    fn run(&mut self, world: &mut World, _event: &GpuWorldSyncEvent) {
        ambient_profiling::scope!("GpuWorldUpdate.run");
        let gpu = world.resource(gpu()).clone();
        world
            .resource_mut(gpu_world())
            .clone()
            .lock()
            .update(&gpu, world);
    }
}
