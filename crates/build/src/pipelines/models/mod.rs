use std::{path::PathBuf, sync::Arc};

use ambient_core::hierarchy::children;
use ambient_ecs::Entity;
use ambient_model_import::{
    model_crate::ModelCrate, MaterialFilter, ModelTextureSize, ModelTransform, TextureResolver,
};
use ambient_physics::collider::{collider_type, ColliderType};
use ambient_pipeline_types::materials::PipelinePbrMaterial;
use ambient_std::asset_url::AssetType;
use futures::FutureExt;
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};

use super::{
    context::PipelineCtx,
    download_image,
    out_asset::{asset_id_from_url, OutAsset, OutAssetContent, OutAssetPreview},
};

pub mod quixel;
pub mod regular;
pub mod unity;

pub async fn pipeline(ctx: &PipelineCtx, config: ModelsPipeline) -> Vec<OutAsset> {
    let mut assets = match &config.importer {
        ModelImporter::Regular => regular::pipeline(ctx, config.clone()).await,
        ModelImporter::UnityModels { use_prefabs } => {
            unity::pipeline(ctx, *use_prefabs, config.clone()).await
        }
        ModelImporter::Quixel => quixel::pipeline(ctx, config.clone()).await,
    };
    if config.collection_of_variants && assets.len() > 1 {
        for asset in &mut assets {
            asset.hidden = true;
        }
        assets.push(OutAsset {
            id: asset_id_from_url(&ctx.out_root().push("col").unwrap()),
            type_: AssetType::Prefab,
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

fn is_true(value: &bool) -> bool {
    *value
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_default<T: PartialEq + Default>(value: &T) -> bool {
    *value == Default::default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelsPipeline {
    /// The importer to use to process models.
    #[serde(default)]
    #[serde(skip_serializing_if = "ModelImporter::is_regular")]
    pub(crate) importer: ModelImporter,
    /// Use assimp as the importer.
    /// This will support more file formats, but is less well-integrated. Off by default.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub(crate) force_assimp: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Collider::is_none")]
    /// The physics collider to use for this mesh.
    pub(crate) collider: Collider,
    /// If a collider is present, this controls how it will interact with other colliders.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub(crate) collider_type: ColliderType,
    /// Whether or not this mesh should have its texture sizes capped.
    pub(crate) cap_texture_sizes: Option<ModelTextureSize>,
    /// Treats all assets in the pipeline as variations, and outputs a single asset which is a collection of all assets.
    /// Most useful for grass and other entities whose individual identity is not important.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub(crate) collection_of_variants: bool,
    /// Output prefabs that can be spawned. On by default.
    #[serde(default = "true_value")]
    #[serde(skip_serializing_if = "is_true")]
    pub(crate) output_prefabs: bool,
    /// Output the animations that belonged to this model.
    #[serde(default = "true_value")]
    #[serde(skip_serializing_if = "is_true")]
    pub(crate) output_animations: bool,
    /// If specified, these components will be added to the prefabs produced by `output_prefabs`.
    ///
    /// This is a great way to specify additional information about your prefab that can be used by gameplay logic.
    /// Note that these components should have static data (i.e. statistics), not dynamic state, as any such state could be
    /// replaced by this prefab being reloaded.
    #[serde(default)]
    #[serde(skip_serializing_if = "Entity::is_empty")]
    pub(crate) prefab_components: Entity,
    /// If specified, a list of overrides to use for the materials for the mesh.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) material_overrides: Vec<MaterialOverride>,
    /// If specified, a list of transformations to apply to this model. This can be used
    /// to correct coordinate space differences between your asset source and the runtime.
    ///
    /// These will be applied in sequence.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) transforms: Vec<ModelTransform>,
}
impl ModelsPipeline {
    async fn apply(
        &self,
        ctx: &PipelineCtx,
        model_crate: &mut ModelCrate,
        out_model_path: impl AsRef<RelativePath>,
    ) -> anyhow::Result<()> {
        for transform in &self.transforms {
            transform.apply(model_crate);
        }
        for mat in &self.material_overrides {
            let material = super::materials::to_mat(
                &mat.material,
                ctx,
                &ctx.in_root(),
                &ctx.out_root()
                    .push(out_model_path.as_ref().join("materials"))?,
            )
            .await?;
            model_crate.override_material(&mat.filter, material);
        }
        if let Some(max_size) = self.cap_texture_sizes {
            model_crate.cap_texture_sizes(max_size.size());
        }
        model_crate.finalize_model();
        match self.collider {
            Collider::None => {}
            Collider::FromModel {
                flip_normals,
                reverse_indices,
            } => {
                model_crate
                    .create_collider_from_model(
                        &ctx.process_ctx.assets,
                        flip_normals,
                        reverse_indices,
                    )
                    .unwrap();
            }
            Collider::Character { radius, height } => {
                model_crate.create_character_collider(radius, height)
            }
        }
        model_crate.add_component_to_prefab(collider_type(), self.collider_type);
        let world = model_crate.prefab_world_mut();
        let obj = world.resource(children())[0];
        world
            .add_components(obj, self.prefab_components.clone())
            .unwrap();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialOverride {
    /// The filter for this override (i.e. what it should apply to).
    pub filter: MaterialFilter,
    /// The material to use as the replacement.
    pub material: PipelinePbrMaterial,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub enum ModelImporter {
    #[default]
    /// The default importer is sufficient for the majority of needs.
    Regular,
    /// Import Unity models.
    UnityModels {
        /// Whether or not the Unity prefabs should be converted to Ambient prefabs.
        use_prefabs: bool,
    },
    /// Import Quixel models.
    Quixel,
}

impl ModelImporter {
    /// Returns `true` if the model importer is [`Regular`].
    ///
    /// [`Regular`]: ModelImporter::Regular
    #[must_use]
    pub fn is_regular(&self) -> bool {
        matches!(self, Self::Regular)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Collider {
    #[default]
    /// No physics collider. The default.
    None,
    /// Extract the physics collider from the model.
    FromModel {
        /// Whether or not the normals should be flipped.
        #[serde(default)]
        #[serde(skip_serializing_if = "is_false")]
        flip_normals: bool,
        /// Whether or not the indices should be reversed for each triangle. On by default.
        #[serde(default = "true_value")]
        #[serde(skip_serializing_if = "is_true")]
        reverse_indices: bool,
    },
    /// Use a cylindrical character collider.
    Character {
        /// The radius of the collider.
        radius: Option<f32>,
        /// The height of the collider.
        height: Option<f32>,
    },
}

impl Collider {
    /// Returns `true` if the collider is [`None`].
    ///
    /// [`None`]: Collider::None
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

fn create_texture_resolver(ctx: &PipelineCtx) -> TextureResolver {
    let ctx = ctx.clone();
    Arc::new(move |path| {
        let ctx = ctx.clone();
        async move {
            let path: PathBuf = path.into();
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            if let Some(file) = ctx
                .files
                .0
                .iter()
                .find(|file| file.decoded_path().as_str().contains(&filename))
            {
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
