use ambient_native_std::asset_url::{AbsAssetUrl, AssetUrl};
use glam::Vec4;
use serde::{Deserialize, Serialize};

use crate::is_false;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
#[serde(deny_unknown_fields)]
pub enum MaterialsImporter {
    /// Import a single material, as specified.
    /// All of its dependent assets (URLs, etc) will be resolved during the build process.
    Single(PipelinePbrMaterial),
    /// Import Quixel materials.
    Quixel,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialsPipeline {
    /// The importer to use for materials.
    pub importer: Box<MaterialsImporter>,
    /// Whether or not decal prefabs should be created for each of these materials.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub output_decals: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
/// A custom pbr material
pub struct PipelinePbrMaterial {
    /// The name of the material.
    pub name: Option<String>,
    /// Where the material came from.
    pub source: Option<String>,

    /// The base color map (i.e. texture) of this material.
    pub base_color: Option<AssetUrl>,
    /// The opacity map of this material.
    pub opacity: Option<AssetUrl>,
    /// The normal map of this material.
    pub normalmap: Option<AssetUrl>,
    /// The metallic roughness map of this material.
    ///
    /// r: metallic
    /// g: roughness
    pub metallic_roughness: Option<AssetUrl>,

    /// The color that this material should be multiplied by. Defaults to white for PBR.
    pub base_color_factor: Option<Vec4>,
    /// The emissive factor of this material (i.e. the color that it emits). Defaults to black for PBR.
    pub emissive_factor: Option<Vec4>,
    /// Whether or not this material is transparent. Defaults to false for PBR.
    pub transparent: Option<bool>,
    /// The opacity level (between 0 and 1) at which this material will not be rendered.
    /// If the opacity map at a point has an opacity lower than this, that point will not be rendered.
    /// Defaults to 0.5 for PBR.
    pub alpha_cutoff: Option<f32>,
    /// Whether or not this material is double-sided. Defaults to false for PBR.
    pub double_sided: Option<bool>,
    /// The metallic coefficient of this material. Defaults to 1 for PBR.
    pub metallic: Option<f32>,
    /// The roughness coefficient of this material. Defaults to 1 for PBR.
    pub roughness: Option<f32>,

    // Non-PBR properties that get translated to PBR.
    /// The non-PBR specular map of this material. If specified, it will be translated to a PBR equivalent.
    pub specular: Option<AssetUrl>,
    /// The non-PBR specular exponent of this material. If specified alongside `specular`, it will be translated to a PBR equivalent.
    pub specular_exponent: Option<f32>,

    /// The sampler used by every texture in this material. Defaults to a sampler with `Linear` min/mag/mip filter modes and `ClampToEdge` wrap modes across uvw-coordinates.
    pub sampler: Option<SamplerKey>,
}

// Quixel

/// Imports a quixel-style surface definition
#[derive(Clone, Debug, Default)]
pub struct QuixelSurfaceDef {
    pub albedo: Option<AbsAssetUrl>,
    pub ao: Option<AbsAssetUrl>,
    pub normal: Option<AbsAssetUrl>,
    pub opacity: Option<AbsAssetUrl>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplerKey {
    pub address_mode_u: wgpu_types::AddressMode,
    pub address_mode_v: wgpu_types::AddressMode,
    pub address_mode_w: wgpu_types::AddressMode,
    pub mag_filter: wgpu_types::FilterMode,
    pub min_filter: wgpu_types::FilterMode,
    pub mipmap_filter: wgpu_types::FilterMode,
}
