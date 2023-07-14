use std::{collections::HashMap, sync::Arc};

use ambient_ecs::{components, ArchetypeId, Resource, System, World};
use ambient_gpu::{gpu::Gpu, shader_module::ShaderModule, typed_buffer::TypedBuffer};
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
use wgpu::{BindGroupLayout, COPY_BUFFER_ALIGNMENT};

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
                resource: buf.buffer.as_entire_binding(),
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
        let buf = self
            .buffers
            .iter()
            .find(|buff| buff.config.format == format)?;
        let comp = buf.layout.get(component)?;
        let offset = *comp.get(&archetype)? as u64;
        Some((&buf.buffer, offset * buf.item_size, buf.layout_version))
    }
}

pub struct GpuComponentsBuffer {
    config: GpuComponentsConfig,
    pub buffer: TypedBuffer<u8>,
    pub layout: HashMap<String, HashMap<ArchetypeId, u32>>,
    layout_offsets: Vec<i32>,
    layout_buffer_offset: u64,
    layout_version: u64,
    item_size: u64,
}

impl GpuComponentsBuffer {
    pub fn new(gpu: &Gpu, config: GpuComponentsConfig) -> Self {
        let size = config.format.size().max(COPY_BUFFER_ALIGNMENT) as usize;
        Self {
            buffer: TypedBuffer::new(
                gpu,
                format!("EntityBuffers.{}.data", config.format),
                size,
                size,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            ),
            layout: config
                .components
                .iter()
                .map(|comp| (comp.name.clone(), HashMap::new()))
                .collect(),
            item_size: config.format.size(),
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
        let mut index = 0;

        for component in &self.config.components {
            let mut layout = HashMap::new();
            for arch in world.archetypes() {
                if component.filter.matches_archetype(arch) {
                    layout.insert(arch.id, index as u32);
                    let buf_len = arch.entity_count();
                    gpu_layout.push(index);
                    let buf_size_pow2 = 2i32.pow((buf_len as f32).log2().ceil() as u32);
                    index += buf_size_pow2;
                } else {
                    gpu_layout.push(-1);
                }
            }
            self.layout.insert(component.name.clone(), layout);
        }

        if gpu_layout != self.layout_offsets
            || index != self.buffer.len() as i32
            || layout_buffer_offset != self.layout_buffer_offset
        {
            self.layout_version += 1;
        }

        self.layout_buffer_offset = layout_buffer_offset;

        self.buffer
            .resize(gpu, index as usize * self.item_size as usize, true);

        layout_buffer.resize(
            gpu,
            gpu_layout.len() + layout_buffer_offset as usize * self.item_size as usize,
            true,
        );

        layout_buffer.write(
            gpu,
            layout_buffer_offset as usize * self.item_size as usize,
            &gpu_layout,
        );
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
