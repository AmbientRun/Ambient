use std::{f32::consts::PI, path::PathBuf, sync::Arc};

use ambient_animation::AnimationOutputs;
use ambient_core::{bounding::local_bounding_aabb, transform::translation};
use ambient_editor_derive::ElementEditor;
use ambient_renderer::materials::pbr_material::PbrMaterialFromUrl;
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    download_asset::{download, AssetsCacheDir},
};
use anyhow::{anyhow, Context};
use async_recursion::async_recursion;
use futures::FutureExt;
use glam::{Mat4, Vec3, Vec4};
use image::RgbaImage;
use model_crate::{ModelCrate, ModelNodeRef};
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};

pub mod assimp;
pub mod fbx;
pub mod gltf;
pub mod model_crate;

pub type TextureResolver = Arc<dyn Fn(String) -> futures::future::BoxFuture<'static, Option<RgbaImage>> + Sync + Send>;

#[derive(Default, Clone, Debug)]
pub struct ModelImportPipeline {
    pub steps: Vec<ModelImportTransform>,
}
impl ModelImportPipeline {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn model(url: AbsAssetUrl) -> Self {
        ModelImportPipeline::new().add_step(ModelImportTransform::ImportModelFromUrl { url, normalize: true, force_assimp: false })
    }
    pub fn model_raw(url: AbsAssetUrl) -> Self {
        ModelImportPipeline::new().add_step(ModelImportTransform::ImportModelFromUrl { url, normalize: false, force_assimp: false })
    }
    pub fn add_step(mut self, step: ModelImportTransform) -> Self {
        self.steps.push(step);
        self
    }
    fn get_cache_path(&self) -> anyhow::Result<String> {
        for step in &self.steps {
            if let ModelImportTransform::ImportModelFromUrl { url, .. } = step {
                return Ok(url.relative_cache_path());
            } else if let ModelImportTransform::MergeMeshLods { lods, .. } = step {
                return Ok(format!("merged_mesh_lods/{}", lods[0].get_cache_path().context("Lod 0 doesn't have a cache path")?));
            } else if let ModelImportTransform::MergeUnityMeshLods { url, .. } = step {
                return Ok(url.relative_cache_path());
            }
        }
        Err(anyhow!("Can't create cache path, no ImportModelFromUrl or MergeMeshLods"))
    }
    pub async fn produce_crate(&self, assets: &AssetCache) -> anyhow::Result<ModelCrate> {
        let mut asset_crate = ModelCrate::new();
        for step in &self.steps {
            step.run(assets, &mut asset_crate).await.with_context(|| format!("Failed to run step: {step:?}"))?;
        }
        Ok(asset_crate)
    }
    pub async fn produce_local_model_url(&self, asset_cache: &AssetCache) -> anyhow::Result<PathBuf> {
        let cache_path = AssetsCacheDir.get(asset_cache).join("pipelines").join(self.get_cache_path()?);
        let model_crate = self.clone().add_step(ModelImportTransform::Finalize).produce_crate(asset_cache).await?;
        model_crate.produce_local_model_url(format!("{}/", cache_path.to_str().unwrap()).into()).await
    }
    // pub async fn produce_local_model(&self, asset_cache: &AssetCache) -> anyhow::Result<Model> {
    //     let url = self.produce_local_model_url(asset_cache).await?;
    //     let mut model = Model::from_file(&url).await?;
    //     model.load(asset_cache).await?;
    //     Ok(model)
    // }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ElementEditor)]
#[serde(tag = "type")]
pub enum MaterialFilter {
    /// Replace all materials.
    All,
    /// Replace all materials that match this name exactly.
    ByName {
        /// The material name to replace. Must match exactly (i.e. is case-sensitive and does not ignore whitespace).
        name: String,
    },
}
impl MaterialFilter {
    pub fn by_name(name: impl Into<String>) -> Self {
        Self::ByName { name: name.into() }
    }
    fn matches(&self, mat: &PbrMaterialFromUrl) -> bool {
        match self {
            MaterialFilter::All => true,
            MaterialFilter::ByName { name } => mat.name.as_ref() == Some(name),
        }
    }
    fn is_all(&self) -> bool {
        matches!(self, MaterialFilter::All)
    }
}
impl Default for MaterialFilter {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Clone, Debug)]
pub enum ModelImportTransform {
    ImportModelFromUrl { url: AbsAssetUrl, normalize: bool, force_assimp: bool },
    MergeMeshLods { lods: Vec<ModelImportPipeline>, lod_cutoffs: Option<Vec<f32>> },
    MergeUnityMeshLods { url: AbsAssetUrl, lod_cutoffs: Option<Vec<f32>> },
    SetName { name: String },
    Transform(ModelTransform),
    OverrideMaterial { filter: MaterialFilter, material: Box<PbrMaterialFromUrl> },
    CapTextureSizes { max_size: ModelTextureSize },
    // RemoveAllMaterials,
    // SetAnimatable { animatable: bool },
    CreatePrefab,
    CreateColliderFromModel,
    CreateCharacterCollider,
    Finalize,
}
impl ModelImportTransform {
    #[async_recursion]
    pub async fn run(&self, assets: &AssetCache, model_crate: &mut ModelCrate) -> anyhow::Result<()> {
        match self {
            ModelImportTransform::ImportModelFromUrl { url, normalize, force_assimp } => {
                model_crate.import(assets, url, *normalize, *force_assimp, Arc::new(|_| async move { None }.boxed())).await?;
            }
            ModelImportTransform::MergeMeshLods { lods, lod_cutoffs } => {
                let mut res_lods = Vec::new();
                for lod in lods {
                    res_lods.push(lod.produce_crate(assets).await?);
                }
                model_crate
                    .merge_mesh_lods(lod_cutoffs.clone(), res_lods.iter().map(|lod| ModelNodeRef { model: lod, root: None }).collect());
            }
            ModelImportTransform::MergeUnityMeshLods { url, lod_cutoffs } => {
                let source = ModelImportPipeline::model(url.clone()).produce_crate(assets).await?;
                model_crate.merge_unity_style_mesh_lods(&source, lod_cutoffs.clone());
            }
            ModelImportTransform::SetName { name } => {
                model_crate.model_world_mut().add_resource(ambient_core::name(), name.clone());
            }
            ModelImportTransform::Transform(transform) => transform.apply(model_crate),
            ModelImportTransform::OverrideMaterial { filter, material } => {
                model_crate.override_material(filter, (**material).clone());
            }
            ModelImportTransform::CapTextureSizes { max_size } => {
                model_crate.cap_texture_sizes(max_size.size());
            }
            // AssetTransform::RemoveAllMaterials => {
            //     model.cpu_materials.clear();
            //     model.gpu_materials.clear();
            // }
            // AssetTransform::SetAnimatable { animatable } => {
            //     model.animatable = Some(*animatable);
            // }
            ModelImportTransform::CreatePrefab => {
                model_crate.create_prefab_from_model();
            }
            ModelImportTransform::CreateColliderFromModel => {
                model_crate.create_collider_from_model(assets, false, true)?;
            }
            ModelImportTransform::CreateCharacterCollider => {
                model_crate.create_character_collider(None, None);
            }
            ModelImportTransform::Finalize => {
                model_crate.finalize_model();
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModelTransform {
    /// Rotate Y up to Z up.
    RotateYUpToZUp,
    /// Rotate X by `deg` degrees.
    RotateX {
        /// The degrees to rotate this model around the X axis.
        deg: f32,
    },
    /// Rotate Y by `deg` degrees.
    RotateY {
        /// The degrees to rotate this model around the Y axis.
        deg: f32,
    },
    /// Rotate Z by `deg` degrees.
    RotateZ {
        /// The degrees to rotate this model around the Z axis.
        deg: f32,
    },
    /// Scale this model.
    Scale {
        /// The factor to scale this model by.
        scale: f32,
    },
    /// Translate this model.
    Translate {
        /// The translation to apply to this model (i.e. this model will be moved by `translation` in the current coordinate space).
        translation: Vec3,
    },
    /// Scale this model's AABB.
    ScaleAABB {
        /// The factor to scale this model's AABB by.
        scale: f32,
    },
    /// Scale this model's animations (spatially, not in time).
    ScaleAnimations {
        /// The factor to scale this model's animations by.
        scale: f32,
    },
    /// Re-root this mesh.
    SetRoot {
        /// The name of the node to set as the new root for this mesh.
        name: String,
    },
    /// Re-center this mesh such that the root is located at the origin.
    Center,
}
impl ModelTransform {
    pub fn apply(&self, model_crate: &mut ModelCrate) {
        match self {
            ModelTransform::RotateYUpToZUp => {
                let transform = Mat4::from_cols(Vec4::X, Vec4::Z, Vec4::Y, Vec4::W);
                model_crate.model_mut().transform(transform);
            }
            ModelTransform::RotateX { deg } => {
                model_crate.model_mut().transform(Mat4::from_rotation_x(deg * PI / 180.));
            }
            ModelTransform::RotateY { deg } => {
                model_crate.model_mut().transform(Mat4::from_rotation_y(deg * PI / 180.));
            }
            ModelTransform::RotateZ { deg } => {
                model_crate.model_mut().transform(Mat4::from_rotation_z(deg * PI / 180.));
            }
            ModelTransform::Scale { scale } => {
                model_crate.model_mut().transform(Mat4::from_scale(Vec3::ONE * *scale));
            }
            ModelTransform::Translate { translation } => {
                model_crate.model_mut().transform(Mat4::from_translation(*translation));
            }
            ModelTransform::ScaleAABB { scale } => {
                let world = model_crate.model_world_mut();
                let aabb = world.resource_mut(local_bounding_aabb());
                aabb.min *= *scale;
                aabb.max *= *scale;
            }
            ModelTransform::ScaleAnimations { scale: anim_scale } => {
                for clip in model_crate.animations.content.values_mut() {
                    *clip = clip.map_outputs(|outputs| {
                        if outputs.component() == translation() {
                            match outputs {
                                AnimationOutputs::Vec3 { component, data } => {
                                    AnimationOutputs::Vec3 { component: *component, data: data.iter().map(|x| *x * *anim_scale).collect() }
                                }
                                AnimationOutputs::Quat { component: _, data: _ } => unreachable!(),
                                AnimationOutputs::Vec3Field { component, field, data } => AnimationOutputs::Vec3Field {
                                    component: *component,
                                    field: *field,
                                    data: data.iter().map(|x| *x * *anim_scale).collect(),
                                },
                            }
                        } else {
                            outputs.clone()
                        }
                    });
                }
            }
            ModelTransform::SetRoot { name } => {
                if let Some(id) = model_crate.model().get_entity_id_by_name(name) {
                    model_crate.make_new_root(id);
                }
            }
            ModelTransform::Center => {
                model_crate.model_mut().center();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ElementEditor)]
pub enum ModelTextureSize {
    /// Cap this model's textures to 128x128.
    X128,
    /// Cap this model's textures to 256x256.
    X256,
    /// Cap this model's textures to 512x512.
    X512,
    /// Cap this model's textures to 1024x1024.
    X1024,
    /// Cap this model's textures to 2048x2048.
    X2048,
    /// Cap this model's textures to 4096x4096.
    X4096,
    /// Cap this model's textures to SIZE x SIZE.
    /// It is strongly recommended that this is a power of two.
    Custom(u32),
}
impl ModelTextureSize {
    pub fn size(&self) -> u32 {
        match self {
            ModelTextureSize::X128 => 128,
            ModelTextureSize::X256 => 256,
            ModelTextureSize::X512 => 512,
            ModelTextureSize::X1024 => 1024,
            ModelTextureSize::X2048 => 2048,
            ModelTextureSize::X4096 => 4096,
            ModelTextureSize::Custom(size) => *size,
        }
    }
}
impl Default for ModelTextureSize {
    fn default() -> Self {
        Self::X512
    }
}

// #[derive(Debug, Clone)]
// pub struct ModelFromAssetPipeline(pub ModelImportPipeline);
// impl ModelFromAssetPipeline {
//     pub fn gltf_file(file: &str) -> Self {
//         Self(ModelImportPipeline::new().add_step(ModelImportTransform::ImportModelFromUrl {
//             url: file.to_string(),
//             normalize: true,
//             force_assimp: false,
//         }))
//     }
// }
// #[async_trait]
// impl AsyncAssetKey<AssetResult<Arc<Model>>> for ModelFromAssetPipeline {
//     async fn load(self, assets: AssetCache) -> AssetResult<Arc<Model>> {
//         Ok(Arc::new(self.0.produce_local_model(&assets).await?))
//     }
// }

pub async fn download_bytes(assets: &AssetCache, url: &AbsAssetUrl) -> anyhow::Result<Vec<u8>> {
    if let Some(path) = url.to_file_path()? {
        Ok(tokio::fs::read(path).await?)
    } else {
        Ok(download(assets, url.0.clone(), |resp| async move { Ok(resp.bytes().await?) }).await?.into())
    }
}

pub const MODEL_EXTENSIONS: &[&str] = &["glb", "fbx", "obj"];

/// ../[path]
pub fn dotdot_path(path: impl Into<RelativePathBuf>) -> RelativePathBuf {
    RelativePathBuf::from("..").join(path.into())
}
pub trait RelativePathBufExt {
    /// [prefix]/[self]
    fn prejoin(&self, prefix: impl Into<RelativePathBuf>) -> Self;
}
impl RelativePathBufExt for RelativePathBuf {
    fn prejoin(&self, prefix: impl Into<RelativePathBuf>) -> Self {
        let prefix: RelativePathBuf = prefix.into();
        prefix.join(self)
    }
}
