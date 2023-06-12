use std::sync::Arc;

use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    shader_module::{BindGroupDesc, ShaderModule},
};
use ambient_renderer::{Material, MaterialShader, SharedMaterial, MATERIAL_BIND_GROUP};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    friendly_id, include_file,
};
use glam::Vec2;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BufferUsages,
};

#[derive(Debug, Clone)]
pub struct GridShaderKey;

fn grid_shader_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: MATERIAL_BIND_GROUP.into(),
    }
}

impl SyncAssetKey<Arc<MaterialShader>> for GridShaderKey {
    fn load(&self, _assets: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            id: "grid_material_shader".to_string(),
            shader: Arc::new(
                ShaderModule::new("GridMaterial", include_file!("grid_material.wgsl"))
                    .with_binding_desc(grid_shader_layout()),
            ),
        })
    }
}

#[repr(C)]
#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct GridMaterialKey {
    pub major: Vec2,
    pub minor: Vec2,
    pub line_width: f32,
    pub size: f32,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct GridMaterial {
    id: String,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl SyncAssetKey<SharedMaterial> for GridMaterialKey {
    fn load(&self, assets: AssetCache) -> SharedMaterial {
        let gpu = GpuKey.get(&assets);
        SharedMaterial::new(GridMaterial::new(&gpu, &assets, *self))
    }
}
impl GridMaterial {
    pub fn new(gpu: &Gpu, assets: &AssetCache, params: GridMaterialKey) -> Self {
        let layout = grid_shader_layout().get(assets);
        let buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("FlatMaterial.buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[params]),
        });
        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                }],
                label: Some("GridMaterial.bind_group"),
            }),
            buffer,
        }
    }
}

impl Material for GridMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn transparent(&self) -> Option<bool> {
        Some(true)
    }
}
