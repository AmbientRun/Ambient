use std::sync::Arc;

use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    shader_module::{BindGroupDesc, ShaderModule},
};
use ambient_renderer::{Material, MaterialShader, RendererShader, SharedMaterial, MATERIAL_BIND_GROUP};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    friendly_id, include_file,
};
use wgpu::{util::DeviceExt, BindGroup};

fn get_loading_layout() -> BindGroupDesc {
    BindGroupDesc {
        entries: vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        }],
        label: MATERIAL_BIND_GROUP.into(),
    }
}

#[derive(Debug, Clone)]
pub struct LoadingShaderKey;

impl SyncAssetKey<Arc<MaterialShader>> for LoadingShaderKey {
    fn load(&self, _assets: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            id: "loading_material_shader".to_string(),
            shader: Arc::new(
                ShaderModule::new("LoadingMaterial", include_file!("loading_material.wgsl")).with_binding_desc(get_loading_layout()),
            ),
        })
    }
}

#[repr(C)]
#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct LoadingMaterialKey {
    pub speed: f32,
    pub scale: f32,
}

impl SyncAssetKey<Arc<RendererShader>> for LoadingMaterialKey {}

#[derive(Debug)]
#[allow(dead_code)]
pub struct LoadingMaterial {
    gpu: Arc<Gpu>,
    id: String,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl SyncAssetKey<SharedMaterial> for LoadingMaterialKey {
    fn load(&self, assets: AssetCache) -> SharedMaterial {
        SharedMaterial::new(LoadingMaterial::new(assets, *self))
    }
}
impl LoadingMaterial {
    pub fn new(assets: AssetCache, params: LoadingMaterialKey) -> Self {
        let gpu = GpuKey.get(&assets);
        let layout = get_loading_layout().get(&assets);

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
                label: Some("LoadingMaterial.bind_group"),
            }),
            buffer,
            gpu: gpu.clone(),
        }
    }
}

impl Material for LoadingMaterial {
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
