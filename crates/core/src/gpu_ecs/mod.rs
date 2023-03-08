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
use ambient_gpu::{gpu::GpuKey, shader_module::BindGroupDesc};
use ambient_std::asset_cache::SyncAssetKeyExt;
pub use component::*;
pub use sync::*;
pub use update::*;

components!("rendering", {
    @[Resource]
    gpu_world: Arc<Mutex<GpuWorld>>,
});

pub const ENTITIES_BIND_GROUP: &str = "ENTITIES_BIND_GROUP";

#[derive(Debug)]
pub struct GpuWorldShaderModuleKey {
    pub read_only: bool,
}

impl SyncAssetKey<ShaderModule> for GpuWorldShaderModuleKey {
    fn load(&self, assets: AssetCache) -> ShaderModule {
        let config = GpuWorldConfigKey.get(&assets);
        let source = config.wgsl(!self.read_only);
        fn entity_component_storage_entry(binding: u32, writeable: bool) -> wgpu::BindGroupLayoutEntry {
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: if writeable {
                    wgpu::ShaderStages::COMPUTE
                } else {
                    wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE
                },
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: !(writeable && binding != 0) },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        }
        let desc = BindGroupDesc {
            entries: (0..(config.buffers.len() + 1)).map(|i| entity_component_storage_entry(i as u32, !self.read_only)).collect_vec(),
            label: ENTITIES_BIND_GROUP.into(),
        };

        ShaderModule::new("GpuWorld", source, vec![desc.into()])
    }
}

pub struct GpuWorld {
    gpu: Arc<Gpu>,
    layout_buffer: TypedBuffer<i32>,
    buffers: Vec<GpuComponentsBuffer>,
    assets: AssetCache,
}
impl GpuWorld {
    pub fn new_arced(assets: AssetCache) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(assets)))
    }
    pub fn new(assets: AssetCache) -> Self {
        let gpu = GpuKey.get(&assets);
        let config = GpuWorldConfigKey.get(&assets);

        log::info!("Creating Gpu Ecs with buffers: {:#?}", config.buffers.iter().map(|v| v.format).collect_vec());

        Self {
            layout_buffer: TypedBuffer::new(
                gpu.clone(),
                "GpuWorld.layout_buffer",
                1,
                1,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ),
            buffers: config.buffers.iter().map(|buffer| GpuComponentsBuffer::new(gpu.clone(), buffer.clone())).collect(),
            assets,
            gpu,
        }
    }
    pub fn update(&mut self, world: &World) {
        self.layout_buffer.write(0, &[world.archetypes().len() as i32]);
        for buf in self.buffers.iter_mut() {
            let layout_offset = buf.config.layout_offset(world.archetypes().len());
            buf.update(world, &mut self.layout_buffer, layout_offset as u64);
        }
    }
    pub fn create_bind_group(&self, read_only: bool) -> wgpu::BindGroup {
        let layout = GpuWorldShaderModuleKey { read_only }.get(&self.assets).idents[0].as_bind_group().unwrap().get(&self.assets);
        let mut buffers = vec![wgpu::BindGroupEntry { binding: 0, resource: self.layout_buffer.buffer().as_entire_binding() }];
        for (i, buf) in self.buffers.iter().enumerate() {
            buffers.push(wgpu::BindGroupEntry { binding: i as u32 + 1, resource: buf.data.buffer.as_entire_binding() });
        }
        self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &buffers,
            label: Some("EntityBuffers.bind_group"),
        })
    }
    pub fn get_buffer(&self, format: GpuComponentFormat, component: &str, archetype: ArchetypeId) -> Option<(&wgpu::Buffer, u64, u64)> {
        let buff = self.buffers.iter().find(|buff| buff.config.format == format)?;
        let comp = buff.layout.get(component)?;
        let offset = *comp.get(&archetype)? as u64;
        Some((&buff.data.buffer, offset * buff.data.item_size(), buff.layout_version))
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
    pub fn new(gpu: Arc<Gpu>, config: GpuComponentsConfig) -> Self {
        Self {
            data: UntypedBuffer::new(
                gpu,
                &format!("EntityBuffers.{}.data", config.format),
                1,
                1,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                config.format.size(),
            ),
            layout: config.components.iter().map(|comp| (comp.name.clone(), HashMap::new())).collect(),
            config,
            layout_offsets: Vec::new(),
            layout_buffer_offset: 0,
            layout_version: 0,
        }
    }
    pub fn update(&mut self, world: &World, layout_buffer: &mut TypedBuffer<i32>, layout_buffer_offset: u64) {
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
        if gpu_layout != self.layout_offsets || offset != self.data.len() as i32 || layout_buffer_offset != self.layout_buffer_offset {
            self.layout_version += 1;
        }
        self.layout_buffer_offset = layout_buffer_offset;
        self.data.resize(offset as u64, true);
        layout_buffer.resize(layout_buffer_offset + gpu_layout.len() as u64, true);
        layout_buffer.write(layout_buffer_offset, &gpu_layout);
        self.layout_offsets = gpu_layout;
    }
}

#[derive(Debug)]
pub struct GpuWorldUpdate;
impl System<GpuWorldSyncEvent> for GpuWorldUpdate {
    fn run(&mut self, world: &mut World, _event: &GpuWorldSyncEvent) {
        profiling::scope!("GpuWorldUpdate.run");
        world.resource_mut(gpu_world()).clone().lock().update(world);
    }
}
