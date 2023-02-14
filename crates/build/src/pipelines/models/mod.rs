use std::{path::PathBuf, sync::Arc};

use futures::FutureExt;
use kiwi_core::hierarchy::children;
use kiwi_ecs::EntityData;
use kiwi_model_import::{model_crate::ModelCrate, MaterialFilter, ModelTextureSize, ModelTransform, TextureResolver};
use kiwi_physics::collider::{collider_type, ColliderType};
use kiwi_std::asset_url::AssetType;
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};

use super::{
    context::PipelineCtx,
    download_image,
    materials::PipelinePbrMaterial,
    out_asset::{asset_id_from_url, OutAsset, OutAssetContent, OutAssetPreview},
};

pub mod quixel;
pub mod regular;
pub mod unity;

pub async fn pipeline(ctx: &PipelineCtx, config: ModelsPipeline) -> Vec<OutAsset> {
    let mut assets = match &config.importer {
        ModelImporter::Regular => regular::pipeline(ctx, config.clone()).await,
        ModelImporter::UnityModels { use_prefabs } => unity::pipeline(ctx, *use_prefabs, config.clone()).await,
        ModelImporter::Quixel => quixel::pipeline(ctx, config.clone()).await,
    };
    if config.collection_of_variants && assets.len() > 1 {
        for asset in &mut assets {
            asset.hidden = true;
        }
        assets.push(OutAsset {
            id: asset_id_from_url(&ctx.out_root().push("col").unwrap()),
            type_: AssetType::Object,
            hidden: false,
            name: ctx.process_ctx.package_name.to_string(),

            tags: Default::default(),
            categories: Default::default(),
            preview: OutAssetPreview::None,
            content: OutAssetContent::Collection(assets.iter().map(|a| a.id.clone()).collect()),
            source: None,
        });
    }
    assets
}

fn true_value() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsPipeline {
    #[serde(default)]
    importer: ModelImporter,
    /// Use assimp as the importer; this will support more file formats, but is less well integrated
    #[serde(default)]
    force_assimp: bool,
    #[serde(default)]
    collider: Collider,
    #[serde(default)]
    collider_type: ColliderType,
    cap_texture_sizes: Option<ModelTextureSize>,
    /// Treats all assets in the pipeline as variations, and outputs a single asset which is a collection of all assets
    #[serde(default)]
    collection_of_variants: bool,
    /// Output objects which can be spawned from server-side scripts
    #[serde(default = "true_value")]
    output_objects: bool,
    #[serde(default = "true_value")]
    output_animations: bool,
    /// Add components to server side objects
    #[serde(default)]
    object_components: EntityData,
    #[serde(default)]
    material_overrides: Vec<MaterialOverride>,
    #[serde(default)]
    transforms: Vec<ModelTransform>,
}
impl ModelsPipeline {
    pub async fn apply(
        &self,
        ctx: &PipelineCtx,
        model_crate: &mut ModelCrate,
        out_model_path: impl AsRef<RelativePath>,
    ) -> anyhow::Result<()> {
        for transform in &self.transforms {
            transform.apply(model_crate);
        }
        for mat in &self.material_overrides {
            let material =
                mat.material.to_mat(ctx, &ctx.in_root(), &ctx.out_root().push(out_model_path.as_ref().join("materials"))?).await?;
            model_crate.override_material(&mat.filter, material);
        }
        if let Some(max_size) = self.cap_texture_sizes {
            model_crate.cap_texture_sizes(max_size.size());
        }
        model_crate.finalize_model();
        match self.collider {
            Collider::None => {}
            Collider::FromModel { flip_normals, reverse_indices } => {
                model_crate.create_collider_from_model(&ctx.process_ctx.assets, flip_normals, reverse_indices).unwrap();
            }
            Collider::Character { radius, height } => model_crate.create_character_collider(radius, height),
        }
        model_crate.add_component_to_object(collider_type(), self.collider_type);
        let world = model_crate.object_world_mut();
        let obj = world.resource(children())[0];
        world.add_components(obj, self.object_components.clone()).unwrap();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialOverride {
    pub filter: MaterialFilter,
    pub material: PipelinePbrMaterial,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum ModelImporter {
    #[default]
    Regular,
    UnityModels {
        use_prefabs: bool,
    },
    Quixel,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum Collider {
    #[default]
    None,
    FromModel {
        #[serde(default)]
        flip_normals: bool,
        #[serde(default)]
        reverse_indices: bool,
    },
    Character {
        radius: Option<f32>,
        height: Option<f32>,
    },
}

fn create_texture_resolver(ctx: &PipelineCtx) -> TextureResolver {
    let ctx = ctx.clone();
    Arc::new(move |path| {
        let ctx = ctx.clone();
        async move {
            let path: PathBuf = path.into();
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            if let Some(file) = ctx.process_ctx.files.iter().find(|file| file.path().as_str().contains(&filename)) {
                match download_image(&ctx.process_ctx.assets, file).await {
                    Ok(img) => Some(img.into_rgba8()),
                    Err(err) => {
                        log::error!("Failed to import image {:?}: {:?}", path, err);
                        None
                    }
                }
            } else {
                None
            }
        }
        .boxed()
    })
}
