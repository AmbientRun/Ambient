use std::sync::Arc;

use glam::Vec2;
use kiwi_gpu::{
    gpu::{Gpu, GpuKey},
    shader_module::ShaderModule,
};
use kiwi_renderer::{Material, MaterialShader, SharedMaterial, MATERIAL_BIND_GROUP};
use kiwi_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    friendly_id, include_file,
};
use wgpu::{util::DeviceExt, BindGroup};

#[derive(Debug, Clone)]
pub struct GridShaderKey;

impl SyncAssetKey<Arc<MaterialShader>> for GridShaderKey {
    fn load(&self, _assets: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            id: "grid_material_shader".to_string(),
            shader: ShaderModule::new(
                "GridMaterial",
                include_file!("grid_material.wgsl"),
                vec![kiwi_gpu::shader_module::BindGroupDesc {
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
                .into()],
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
    gpu: Arc<Gpu>,
    id: String,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl SyncAssetKey<SharedMaterial> for GridMaterialKey {
    fn load(&self, assets: AssetCache) -> SharedMaterial {
        SharedMaterial::new(GridMaterial::new(assets, *self))
    }
}
impl GridMaterial {
    pub fn new(assets: AssetCache, params: GridMaterialKey) -> Self {
        let gpu = GpuKey.get(&assets);
        let layout = GridShaderKey.get(&assets).shader.first_layout(&assets);
        let buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FlatMaterial.buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[params]),
        });
        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()) }],
                label: Some("GridMaterial.bind_group"),
            }),
            buffer,
            gpu: gpu.clone(),
        }
    }
}

impl Material for GridMaterial {
    fn bind(&self) -> &BindGroup {
        &self.bind_group
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn transparent(&self) -> Option<bool> {
        Some(true)
    }
}
