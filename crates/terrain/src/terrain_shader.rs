use std::sync::Arc;

use ambient_core::gpu_ecs::ENTITIES_BIND_GROUP;
use anyhow::Context;
use async_trait::async_trait;

use ambient_ecs::World;
use ambient_editor_derive::ElementEditor;
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    shader_module::{BindGroupDesc, Shader, ShaderIdent, ShaderModule, WgslValue},
    std_assets::DefaultSamplerKey,
    texture::{Texture, TextureView},
    texture_loaders::TextureArrayFromUrls,
};
use ambient_renderer::{
    materials::pbr_material::PbrMaterialDesc, Material, RendererShader, GLOBALS_BIND_GROUP, MATERIAL_BIND_GROUP, PRIMITIVES_BIND_GROUP,
    RESOURCES_BIND_GROUP,
};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, MaterialAssetType, TypedAssetUrl},
    download_asset::{AssetError, JsonFromUrl},
    friendly_id, include_file,
};
use futures::future::join_all;
use glam::{UVec2, Vec3};
use serde::{Deserialize, Serialize};
use wgpu::{util::DeviceExt, BindGroup};

use crate::{TerrainLayers, TERRAIN_BASE};

fn get_terrain_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![
            // terrain_params
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            // heightmap_sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // heightmap
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            // normalmap
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
            // base_colors
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            // texture_normals
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            // texture_sampler
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // terrain_mat_def
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            // noise_texture
            wgpu::BindGroupLayoutEntry {
                binding: 8,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
        label: MATERIAL_BIND_GROUP.into(),
    }
}

#[derive(Debug, Clone)]
pub struct TerrainShaderKey {
    pub shadow_cascades: u32,
}
impl SyncAssetKey<Arc<RendererShader>> for TerrainShaderKey {
    fn load(&self, assets: AssetCache) -> Arc<RendererShader> {
        let shader = Shader::new(
            &assets,
            "terrrain shader",
            &[GLOBALS_BIND_GROUP, ENTITIES_BIND_GROUP, RESOURCES_BIND_GROUP, PRIMITIVES_BIND_GROUP, MATERIAL_BIND_GROUP],
            &ShaderModule::new("Terrain", include_file!("./terrain.wgsl"))
                .with_binding_desc(get_terrain_layout())
                .with_ident(ShaderIdent::constant("TERRAIN_FUNCS", WgslValue::Raw(include_str!("terrain_funcs.wgsl").into())))
                .with_ident(ShaderIdent::constant("GET_HARDNESS", WgslValue::Raw(include_str!("brushes/get_hardness.wgsl").into())))
                .with_ident(ShaderIdent::constant("ROCK_LAYER", TerrainLayers::Rock as u32))
                .with_ident(ShaderIdent::constant("SOIL_LAYER", TerrainLayers::Soil as u32))
                .with_ident(ShaderIdent::constant("SEDIMENT_LAYER", TerrainLayers::Sediment as u32))
                .with_ident(ShaderIdent::constant("WATER_LAYER", TerrainLayers::Water as u32))
                .with_ident(ShaderIdent::constant("WATER_OUTFLOW_L_LAYER", TerrainLayers::WaterOutflowL as u32))
                .with_ident(ShaderIdent::constant("WATER_OUTFLOW_R_LAYER", TerrainLayers::WaterOutflowR as u32))
                .with_ident(ShaderIdent::constant("WATER_OUTFLOW_T_LAYER", TerrainLayers::WaterOutflowT as u32))
                .with_ident(ShaderIdent::constant("WATER_OUTFLOW_B_LAYER", TerrainLayers::WaterOutflowB as u32))
                .with_ident(ShaderIdent::constant("WATER_VELOCITY_X_LAYER", TerrainLayers::WaterVelocityX as u32))
                .with_ident(ShaderIdent::constant("WATER_VELOCITY_Y_LAYER", TerrainLayers::WaterVelocityY as u32))
                .with_ident(ShaderIdent::constant("HARDNESS_LAYER", TerrainLayers::Hardness as u32))
                .with_ident(ShaderIdent::constant("HARDNESS_STRATA_AMOUNT_LAYER", TerrainLayers::HardnessStrataAmount as u32))
                .with_ident(ShaderIdent::constant("#HARDNESS_STRATA_WAVELENGTH_LAYER", TerrainLayers::HardnessStrataWavelength as u32))
                .with_ident(ShaderIdent::constant("#TERRAIN_BASE", TERRAIN_BASE)),
        )
        .unwrap();

        Arc::new(RendererShader {
            shader,
            id: "TerrainShader".to_string(),
            vs_main: "vs_main".to_string(),
            fs_forward_main: "fs_forward_main".to_string(),
            fs_shadow_main: "fs_shadow_main".to_string(),
            fs_outline_main: "fs_outlines_main".to_string(),
            transparent: false,
            double_sided: false,
            depth_write_enabled: true,
            transparency_group: 0,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainMaterialParams {
    pub heightmap_position: Vec3,
    pub lod_factor: f32,
    pub cell_diagonal: f32,
    pub _padding: Vec3,
}
#[derive(Debug)]
pub struct TerrainMaterial {
    gpu: Arc<Gpu>,
    id: String,
    pub params: TerrainMaterialParams,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
impl TerrainMaterial {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assets: AssetCache,
        params: TerrainMaterialParams,
        heightmap: Arc<TextureView>,
        normalmap: Arc<TextureView>,
        base_colors: Arc<TextureView>,
        texture_normals: Arc<TextureView>,
        noise_texture: Arc<TextureView>,
        material_def: TerrainMaterialDef,
    ) -> Self {
        let gpu = GpuKey.get(&assets);
        let layout = get_terrain_layout().get(&assets);

        let params_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TerrainMaterial.params_buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[params]),
        });
        let mat_build = material_def.build();
        let def_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TerrainMaterial.def_buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[mat_build.params]),
        });
        let heightmap_sampler = Arc::new(gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }));
        let default_sampler = DefaultSamplerKey.get(&assets);
        Self {
            id: friendly_id(),
            params,
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(params_buffer.as_entire_buffer_binding()) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&heightmap_sampler) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&heightmap) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&normalmap) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&base_colors) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&texture_normals) },
                    wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&default_sampler) },
                    wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::Buffer(def_buffer.as_entire_buffer_binding()) },
                    wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::TextureView(&noise_texture) },
                ],
                label: Some("TerrainMaterial.bind_group"),
            }),
            buffer: params_buffer,
            // surface_colors,
            // noise_texture,
            gpu: gpu.clone(),
        }
    }
}
impl Material for TerrainMaterial {
    fn update(&self, _world: &World) {
        let params = self.params;
        self.gpu.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[params]));
    }
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct TerrainTexturesKey {
    pub texs: Vec<TypedAssetUrl<MaterialAssetType>>,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<Texture>, AssetError>> for TerrainTexturesKey {
    fn gpu_size(&self, asset: &Result<Arc<Texture>, AssetError>) -> Option<u64> {
        asset.as_ref().ok().map(|asset| asset.size_in_bytes)
    }
    async fn load(self, assets: AssetCache) -> Result<Arc<Texture>, AssetError> {
        let color_urls: Vec<Result<AbsAssetUrl, AssetError>> = join_all(
            self.texs
                .into_iter()
                .map(|tex| {
                    let assets = assets.clone();
                    async move {
                        let mat_url = tex.abs().context("Not an absolute url")?;
                        let mat: Arc<PbrMaterialDesc> = JsonFromUrl::new(mat_url.clone(), true).get(&assets).await?;
                        let mat = mat.resolve(&mat_url)?;
                        Ok(mat.base_color.as_ref().unwrap().clone().unwrap_abs())
                    }
                })
                .collect::<Vec<_>>(),
        )
        .await;
        let color_urls = color_urls.into_iter().collect::<Result<Vec<_>, _>>()?;

        let colors = TextureArrayFromUrls {
            urls: color_urls,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            label: Some("GameTerrain.ground_textures".to_string()),
        }
        .get(&assets)
        .await?;
        // let normals = TextureArrayFromUrls {
        //     urls: texs
        //         .iter()
        //         .map(|(uid, _, normal, _)| {
        //             url(format!("QuixelSurfaces/{uid}/{}", normal))
        //             // Url::parse(&format!("{CONTENT_SERVER_URL}assets/models/Misc/TestNormalMap2k.png")).unwrap()
        //         })
        //         .collect_vec(),
        //     format: wgpu::TextureFormat::Rgba8Unorm,
        //     label: Some("GameTerrain.ground_textures".to_string()),
        // }
        // .get(assets)
        // .await
        // .unwrap();
        Ok(colors)
    }
}

pub enum TerrainPreset {
    Mountains,
    Desert,
    LowPerformanceMode,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TerrainMaterialDef {
    #[serde(default)]
    pub settings: TerrainWGSLMatSettings,
    #[serde(default)]
    pub soft_rock1: TerrainSurface,
    #[serde(default)]
    pub soft_rock2: TerrainSurface,
    #[serde(default)]
    pub hard_rock1: TerrainSurface,
    #[serde(default)]
    pub hard_rock2: TerrainSurface,
    #[serde(default)]
    pub forest_floor1: TerrainSurface,
    #[serde(default)]
    pub forest_floor2: TerrainSurface,
    #[serde(default)]
    pub grass1: TerrainSurface,
    #[serde(default)]
    pub grass2: TerrainSurface,
    #[serde(default)]
    pub sand: TerrainSurface,
}
impl TerrainMaterialDef {
    pub fn load(preset: TerrainPreset) -> Self {
        match preset {
            TerrainPreset::Mountains => serde_json::from_str(&include_file!("mountains.json")).unwrap(),
            TerrainPreset::Desert => serde_json::from_str(&include_file!("desert.json")).unwrap(),
            TerrainPreset::LowPerformanceMode => serde_json::from_str(&include_file!("mountains.json")).unwrap(),
        }
    }
    pub fn build(&self) -> TerrainMaterialBuild {
        let mut res = TerrainMaterialBuild::default();
        res.params.soft_rock1 = self.soft_rock1.build(&mut res);
        res.params.soft_rock2 = self.soft_rock2.build(&mut res);
        res.params.hard_rock1 = self.hard_rock1.build(&mut res);
        res.params.hard_rock2 = self.hard_rock2.build(&mut res);
        res.params.forest_floor1 = self.forest_floor1.build(&mut res);
        res.params.forest_floor2 = self.forest_floor2.build(&mut res);
        res.params.grass1 = self.grass1.build(&mut res);
        res.params.grass2 = self.grass2.build(&mut res);
        res.params.sand = self.sand.build(&mut res);
        res.params.settings = self.settings;
        res
    }
}
#[derive(Default)]
pub struct TerrainMaterialBuild {
    pub textures: Vec<TypedAssetUrl<MaterialAssetType>>,
    pub params: TerrainWGSLMat,
}
impl TerrainMaterialBuild {
    fn insert_texture(&mut self, texture: TypedAssetUrl<MaterialAssetType>) -> usize {
        if let Some(x) = self.textures.iter().position(|x| x == &texture) {
            x
        } else {
            let x = self.textures.len();
            self.textures.push(texture);
            x
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TerrainSurface {
    top_texture: TypedAssetUrl<MaterialAssetType>,
    side_texture: TypedAssetUrl<MaterialAssetType>,
    settings: TerrainTriplanarSettings,
}
impl TerrainSurface {
    fn build(&self, res: &mut TerrainMaterialBuild) -> TerrainWGSLTriplanarSample {
        let top = res.insert_texture(self.top_texture.clone()) as i32;
        let side = res.insert_texture(self.side_texture.clone()) as i32;
        TerrainWGSLTriplanarSample { settings: self.settings, top_texture: top, side_texture: side, _padding: Default::default() }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainWGSLMat {
    soft_rock1: TerrainWGSLTriplanarSample,
    soft_rock2: TerrainWGSLTriplanarSample,
    hard_rock1: TerrainWGSLTriplanarSample,
    hard_rock2: TerrainWGSLTriplanarSample,
    forest_floor1: TerrainWGSLTriplanarSample,
    forest_floor2: TerrainWGSLTriplanarSample,
    grass1: TerrainWGSLTriplanarSample,
    grass2: TerrainWGSLTriplanarSample,
    sand: TerrainWGSLTriplanarSample,
    settings: TerrainWGSLMatSettings,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, ElementEditor, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainWGSLMatSettings {
    #[serde(default)]
    #[editor(slider, min = 0., max = 40.)]
    grass_depth: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 10.)]
    grass_gradient: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 10.)]
    rock_soil_gradient: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 10.)]
    beach_gradient: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 200.)]
    beach_noise_scale: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 1.)]
    beach_noise_steepness: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 500.)]
    variation_texture_scale: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 1.)]
    variation_gradient: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainWGSLTriplanarSample {
    settings: TerrainTriplanarSettings,
    top_texture: i32,
    side_texture: i32,
    _padding: UVec2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ElementEditor, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainTriplanarSettings {
    top_color: Vec3,
    rot_top_45_deg: u32,
    side_color: Vec3,
    rot_side_45_deg: u32,
    #[editor(slider, min = 0.1, max = 10.)]
    top_scale: f32,
    #[editor(slider, min = 0.1, max = 10.)]
    side_scale: f32,
    #[editor(slider, min = 0.1, max = 50.)]
    hardness: f32,
    #[editor(slider, min = 0.1, max = 50.)]
    angle: f32,
}
impl Default for TerrainTriplanarSettings {
    fn default() -> Self {
        Self {
            top_scale: 0.5,
            side_scale: 0.5,
            top_color: Vec3::ONE,
            side_color: Vec3::ONE,
            hardness: 1.,
            angle: 1.,
            rot_top_45_deg: 0,
            rot_side_45_deg: 0,
        }
    }
}
