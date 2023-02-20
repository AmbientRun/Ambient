use std::sync::Arc;

use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    shader_module::BindGroupDesc,
    std_assets::DefaultSamplerKey,
    texture::TextureView,
};
use ambient_renderer::{Material, MaterialShader, RendererConfig, RendererShader, StandardShaderKey, MATERIAL_BIND_GROUP};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    friendly_id, include_file,
};
use wgpu::BindGroup;

#[derive(Debug, Clone)]
pub struct TextMaterialShaderKey;
impl SyncAssetKey<Arc<MaterialShader>> for TextMaterialShaderKey {
    fn load(&self, _: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            id: "text_material_shader".to_string(),
            shader: ambient_gpu::shader_module::ShaderModule::new(
                "TextMaterial",
                include_file!("text_material.wgsl"),
                vec![BindGroupDesc {
                    entries: vec![
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: MATERIAL_BIND_GROUP.into(),
                }
                .into()],
            ),
        })
    }
}

pub fn get_text_shader(assets: &AssetCache, config: &RendererConfig) -> Arc<RendererShader> {
    StandardShaderKey { material_shader: TextMaterialShaderKey.get(assets), lit: false, shadow_cascades: config.shadow_cascades }
        .get(assets)
}

pub struct TextMaterial {
    _gpu: Arc<Gpu>,
    id: String,
    bind_group: wgpu::BindGroup,
}
impl TextMaterial {
    pub fn new(assets: AssetCache, font_atlas: Arc<TextureView>) -> Self {
        let gpu = GpuKey.get(&assets);
        let material = TextMaterialShaderKey.get(&assets);
        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &material.shader.first_layout(&assets),
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&font_atlas) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&DefaultSamplerKey.get(&assets)) },
                ],
                label: Some("TextMaterial.bind_group"),
            }),
            _gpu: gpu.clone(),
        }
    }
}
impl std::fmt::Debug for TextMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextMaterial").field("id", &self.id).finish()
    }
}
impl Material for TextMaterial {
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
