use std::{path::PathBuf, sync::Arc};

use ambient_core::hierarchy::children;
use ambient_ecs::Entity;
use ambient_model_import::{
    model_crate::ModelCrate, MaterialFilter, ModelTextureSize, ModelTransform, TextureResolver,
};
use ambient_physics::collider::{collider_type, ColliderType};
use ambient_pipeline_types::{
    materials::PipelinePbrMaterial,
    models::{Collider, ModelImporter, ModelsPipeline},
};
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

async fn apply(
    pipeline: &ModelsPipeline,
    ctx: &PipelineCtx,
    model_crate: &mut ModelCrate,
    out_model_path: impl AsRef<RelativePath>,
) -> anyhow::Result<()> {
    for transform in &pipeline.transforms {
        transform.apply(model_crate);
    }
    for mat in &pipeline.material_overrides {
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
    if let Some(max_size) = pipeline.cap_texture_sizes {
        model_crate.cap_texture_sizes(max_size.size());
    }
    model_crate.finalize_model();
    match pipeline.collider {
        Collider::None => {}
        Collider::FromModel {
            flip_normals,
            reverse_indices,
        } => {
            model_crate
                .create_collider_from_model(&ctx.process_ctx.assets, flip_normals, reverse_indices)
                .unwrap();
        }
        Collider::Character { radius, height } => {
            model_crate.create_character_collider(radius, height)
        }
    }
    model_crate.add_component_to_prefab(collider_type(), pipeline.collider_type);
    let world = model_crate.prefab_world_mut();
    let obj = world.resource(children())[0];
    world
        .add_components(obj, pipeline.prefab_components.clone())
        .unwrap();
    Ok(())
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
