use std::sync::Arc;

use glam::Vec4;
use kiwi_gpu::{
    gpu::{Gpu, GpuKey}, shader_module::{BindGroupDesc, ShaderModule}
};
use kiwi_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt}, friendly_id, include_file
};
use wgpu::{util::DeviceExt, BindGroup};

use super::super::{Material, MaterialShader, RendererShader, MATERIAL_BIND_GROUP};
use crate::{SharedMaterial, StandardShaderKey};

#[derive(Debug)]
pub struct FlatMaterialShaderKey;
impl SyncAssetKey<Arc<MaterialShader>> for FlatMaterialShaderKey {
    fn load(&self, _assets: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            shader: ShaderModule::new(
                "FlatMaterial",
                include_file!("flat_material.wgsl"),
                vec![BindGroupDesc {
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

            id: "flat_material_shader".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FlatShaderKey {
    pub lit: bool,
}

impl SyncAssetKey<Arc<RendererShader>> for FlatShaderKey {
    fn load(&self, assets: AssetCache) -> Arc<RendererShader> {
        StandardShaderKey { material_shader: FlatMaterialShaderKey.get(&assets), lit: self.lit }.get(&assets)
    }
}

pub fn get_flat_shader(assets: &AssetCache) -> Arc<RendererShader> {
    StandardShaderKey { material_shader: FlatMaterialShaderKey.get(assets), lit: true }.get(assets)
}

pub fn get_flat_shader_unlit(assets: &AssetCache) -> Arc<RendererShader> {
    StandardShaderKey { material_shader: FlatMaterialShaderKey.get(assets), lit: false }.get(assets)
}

#[derive(Debug)]
pub struct FlatMaterialKey {
    color: Vec4,
    transparent: Option<bool>,
}
impl FlatMaterialKey {
    pub fn new(color: Vec4, transparent: Option<bool>) -> Self {
        Self { color, transparent }
    }

    pub fn white() -> Self {
        Self { color: Vec4::ONE, transparent: Some(false) }
    }
    pub fn transparent() -> Self {
        Self { color: Vec4::ONE, transparent: Some(true) }
    }
}
impl SyncAssetKey<SharedMaterial> for FlatMaterialKey {
    fn load(&self, assets: AssetCache) -> SharedMaterial {
        SharedMaterial::new(FlatMaterial::new(assets, self.color, self.transparent))
    }
}

pub struct FlatMaterial {
    gpu: Arc<Gpu>,
    id: String,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    _color: Vec4,
    transparent: Option<bool>,
}
impl FlatMaterial {
    pub fn new(assets: AssetCache, color: Vec4, transparent: Option<bool>) -> Self {
        let gpu = GpuKey.get(&assets);
        let layout = FlatMaterialShaderKey.get(&assets).shader.first_layout(&assets);

        let buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FlatMaterial.buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[color]),
        });
        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()) }],
                label: Some("FlatMaterial.bind_group"),
            }),
            buffer,
            gpu: gpu.clone(),
            _color: color,
            transparent,
        }
    }
    pub fn update_color(&self, color: Vec4) {
        self.gpu.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[color]));
    }
}
impl std::fmt::Debug for FlatMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlatMaterial").field("id", &self.id).finish()
    }
}
impl Material for FlatMaterial {
    fn bind(&self) -> &BindGroup {
        &self.bind_group
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn transparent(&self) -> Option<bool> {
        self.transparent
    }
}
