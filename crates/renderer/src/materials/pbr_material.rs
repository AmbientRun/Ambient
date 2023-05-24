use std::sync::Arc;

use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    shader_module::{BindGroupDesc, ShaderModule},
    std_assets::{DefaultNormalMapViewKey, DefaultSamplerKey, PixelTextureViewKey},
    texture::{Texture, TextureView},
    texture_loaders::{SplitTextureFromUrl, TextureFromUrl},
};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, AssetUrl},
    download_asset::{AssetError, JsonFromUrl},
    friendly_id, include_file,
};
use async_trait::async_trait;
use glam::Vec4;
use serde::{Deserialize, Serialize};
use wgpu::{util::DeviceExt, BindGroup};

use super::super::{Material, MaterialShader, RendererShader, MATERIAL_BIND_GROUP};
use crate::{RendererConfig, StandardShaderKey};

fn get_material_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        label: MATERIAL_BIND_GROUP.into(),
        entries: vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    }
}

#[derive(Debug)]
pub struct PbrMaterialShaderKey;
impl SyncAssetKey<Arc<MaterialShader>> for PbrMaterialShaderKey {
    fn load(&self, _assets: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            id: "pbr_material_shader".to_string(),
            shader: Arc::new(
                ShaderModule::new("PbrMaterial", include_file!("pbr_material.wgsl"))
                    .with_binding_desc(get_material_layout()),
            ),
        })
    }
}

pub fn get_pbr_shader(assets: &AssetCache, config: &RendererConfig) -> Arc<RendererShader> {
    StandardShaderKey {
        material_shader: PbrMaterialShaderKey.get(assets),
        lit: true,
        shadow_cascades: config.shadow_cascades,
    }
    .get(assets)
}

pub fn get_pbr_shader_unlit(assets: &AssetCache, config: &RendererConfig) -> Arc<RendererShader> {
    StandardShaderKey {
        material_shader: PbrMaterialShaderKey.get(assets),
        lit: false,
        shadow_cascades: config.shadow_cascades,
    }
    .get(assets)
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PbrMaterialParams {
    pub base_color_factor: Vec4,
    pub emissive_factor: Vec4,
    pub alpha_cutoff: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub _padding: u32,
}
impl Default for PbrMaterialParams {
    fn default() -> Self {
        Self {
            base_color_factor: Vec4::ONE,
            emissive_factor: Vec4::ZERO,
            alpha_cutoff: 0.5,
            metallic: 1.0,
            roughness: 1.0,
            _padding: Default::default(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct PbrMaterialConfig {
    pub source: String,
    pub name: String,
    pub params: PbrMaterialParams,
    pub base_color: Arc<TextureView>,
    pub normalmap: Arc<TextureView>,
    /// r: Metallic, g: Roughness
    pub metallic_roughness: Arc<TextureView>,
    pub sampler: Arc<wgpu::Sampler>,
    pub transparent: bool,
    pub double_sided: bool,
    pub depth_write_enabled: bool,
}

pub struct PbrMaterial {
    gpu: Arc<Gpu>,
    id: String,
    pub config: PbrMaterialConfig,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl PbrMaterial {
    pub fn new(assets: &AssetCache, config: PbrMaterialConfig) -> Self {
        let gpu = GpuKey.get(assets);
        let layout = get_material_layout().get(assets);

        let buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("PbrMaterial.buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&[config.params]),
            });

        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&config.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&config.base_color.handle),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&config.normalmap.handle),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(
                            &config.metallic_roughness.handle,
                        ),
                    },
                ],
                label: Some("PbrMaterial.bind_group"),
            }),
            buffer,
            gpu: gpu.clone(),
            config,
        }
    }
    pub fn base_color_from_file(assets: &AssetCache, url: &str) -> Self {
        let texture = Arc::new(
            Arc::new(Texture::from_file(
                GpuKey.get(assets),
                url,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ))
            .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        PbrMaterial::new(
            assets,
            PbrMaterialConfig {
                source: url.to_string(),
                name: url.to_string(),
                params: PbrMaterialParams::default(),
                base_color: texture,
                normalmap: DefaultNormalMapViewKey.get(assets),
                metallic_roughness: PixelTextureViewKey::white().get(assets),
                sampler: DefaultSamplerKey.get(assets),
                transparent: false,
                double_sided: false,
                depth_write_enabled: false,
            },
        )
    }
    pub fn upload_params(&self) {
        self.gpu
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.config.params]));
    }
    pub fn gpu_size(&self) -> u64 {
        self.config.base_color.texture.size_in_bytes
            + self.config.normalmap.texture.size_in_bytes
            + self.config.metallic_roughness.texture.size_in_bytes
    }
}
impl std::fmt::Debug for PbrMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PbrMaterial")
            .field("id", &self.id)
            .field("source", &self.config.source)
            .field("name", &self.config.name)
            .finish()
    }
}
impl Material for PbrMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.config.name
    }
    fn transparent(&self) -> Option<bool> {
        Some(self.config.transparent)
    }
    fn double_sided(&self) -> Option<bool> {
        Some(self.config.double_sided)
    }
    fn depth_write_enabled(&self) -> Option<bool> {
        Some(self.config.depth_write_enabled)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}

impl Default for AlphaMode {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PbrMaterialFromUrl(pub AbsAssetUrl);
#[async_trait]
impl AsyncAssetKey<Result<Arc<PbrMaterial>, AssetError>> for PbrMaterialFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<PbrMaterial>, AssetError> {
        let mat_def = JsonFromUrl::<PbrMaterialDesc>::new(self.0.clone(), true)
            .get(&assets)
            .await?;
        let mat = mat_def.resolve(&self.0)?.get(&assets).await?;
        Ok(mat)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PbrMaterialDesc {
    pub name: Option<String>,
    pub source: Option<String>,

    pub base_color: Option<AssetUrl>,
    pub opacity: Option<AssetUrl>,
    pub normalmap: Option<AssetUrl>,
    pub metallic_roughness: Option<AssetUrl>,

    pub base_color_factor: Option<Vec4>,
    pub emissive_factor: Option<Vec4>,
    pub transparent: Option<bool>,
    pub alpha_cutoff: Option<f32>,
    pub double_sided: Option<bool>,
    #[serde(default)]
    pub metallic: f32,
    #[serde(default)]
    pub roughness: f32,
}
impl PbrMaterialDesc {
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> anyhow::Result<Self> {
        Ok(Self {
            name: self.name.clone(),
            source: self.source.clone(),

            base_color: if let Some(x) = &self.base_color {
                Some(x.resolve(base_url)?.into())
            } else {
                None
            },
            opacity: if let Some(x) = &self.opacity {
                Some(x.resolve(base_url)?.into())
            } else {
                None
            },
            normalmap: if let Some(x) = &self.normalmap {
                Some(x.resolve(base_url)?.into())
            } else {
                None
            },
            metallic_roughness: if let Some(x) = &self.metallic_roughness {
                Some(x.resolve(base_url)?.into())
            } else {
                None
            },

            base_color_factor: self.base_color_factor,
            emissive_factor: self.emissive_factor,
            transparent: self.transparent,
            alpha_cutoff: self.alpha_cutoff,
            double_sided: self.double_sided,
            metallic: self.metallic,
            roughness: self.roughness,
        })
    }
    pub fn relative_path_from(&self, base_url: &AbsAssetUrl) -> Self {
        Self {
            name: self.name.clone(),
            source: self.source.clone(),

            base_color: self
                .base_color
                .as_ref()
                .map(|x| base_url.relative_path(x.path()).into()),
            opacity: self
                .opacity
                .as_ref()
                .map(|x| base_url.relative_path(x.path()).into()),
            normalmap: self
                .normalmap
                .as_ref()
                .map(|x| base_url.relative_path(x.path()).into()),
            metallic_roughness: self
                .metallic_roughness
                .as_ref()
                .map(|x| base_url.relative_path(x.path()).into()),

            base_color_factor: self.base_color_factor,
            emissive_factor: self.emissive_factor,
            transparent: self.transparent,
            alpha_cutoff: self.alpha_cutoff,
            double_sided: self.double_sided,
            metallic: self.metallic,
            roughness: self.roughness,
        }
    }
}

#[async_trait]
impl AsyncAssetKey<Result<Arc<PbrMaterial>, AssetError>> for PbrMaterialDesc {
    async fn load(self, assets: AssetCache) -> Result<Arc<PbrMaterial>, AssetError> {
        let color = if let (Some(opacity), Some(albedo)) = (&self.opacity, &self.base_color) {
            Some(
                SplitTextureFromUrl {
                    color: albedo.clone().unwrap_abs(),
                    alpha: opacity.clone().unwrap_abs(),
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                }
                .get(&assets)
                .await?,
            )
        } else if let Some(albedo) = &self.base_color {
            Some(
                TextureFromUrl {
                    url: albedo.clone().unwrap_abs(),
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                }
                .get(&assets)
                .await?,
            )
        } else {
            None
        };
        let color_view = match color {
            Some(color) => Arc::new(color.create_view(&wgpu::TextureViewDescriptor::default())),
            None => PixelTextureViewKey::white().get(&assets),
        };
        let normalmap = if let Some(normalmap) = &self.normalmap {
            Arc::new(
                TextureFromUrl {
                    url: normalmap.clone().unwrap_abs(),
                    format: wgpu::TextureFormat::Rgba8Unorm,
                }
                .get(&assets)
                .await?
                .create_view(&Default::default()),
            )
        } else {
            DefaultNormalMapViewKey.get(&assets)
        };

        let metallic_roughness = if let Some(metallic_roughness) = self.metallic_roughness {
            Arc::new(
                TextureFromUrl {
                    url: metallic_roughness.clone().unwrap_abs(),
                    format: wgpu::TextureFormat::Rgba8Unorm,
                }
                .get(&assets)
                .await?
                .create_view(&Default::default()),
            )
        } else {
            PixelTextureViewKey::white().get(&assets)
        };

        let sampler = DefaultSamplerKey.get(&assets);

        let params = PbrMaterialParams {
            base_color_factor: self.base_color_factor.unwrap_or(Vec4::ONE),
            emissive_factor: self.emissive_factor.unwrap_or(Vec4::ZERO),
            alpha_cutoff: self.alpha_cutoff.unwrap_or(0.01),
            metallic: self.metallic,
            roughness: self.roughness,
            _padding: Default::default(),
        };

        let name = self
            .name
            .or(self.base_color.map(|x| x.to_string()))
            .unwrap_or_default();
        Ok(Arc::new(PbrMaterial::new(
            &assets,
            PbrMaterialConfig {
                source: self.source.unwrap_or_default(),
                name,
                params,
                base_color: color_view.clone(),
                normalmap,
                metallic_roughness,
                sampler,
                transparent: self.transparent.unwrap_or_default(),
                double_sided: self.double_sided.unwrap_or_default(),
                depth_write_enabled: false,
            },
        )))
    }
}
