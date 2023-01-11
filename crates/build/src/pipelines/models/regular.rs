use anyhow::Context;
use elements_model_import::model_crate::ModelCrate;
use elements_std::asset_url::AssetType;

use super::{
    super::{
        context::PipelineCtx, out_asset::{OutAssetContent, OutAssetPreview}, ModelsPipeline
    }, create_texture_resolver
};
use crate::pipelines::OutAsset;

pub async fn pipeline(ctx: &PipelineCtx, config: ModelsPipeline) -> Vec<OutAsset> {
    ctx.process_files(
        |f| {
            f.sub_path.extension == Some("fbx".to_string())
                || f.sub_path.extension == Some("glb".to_string())
                || f.sub_path.extension == Some("obj".to_string())
        },
        move |ctx, file| {
            let config = config.clone();
            async move {
                let mut res = Vec::new();

                let asset_crate_id = ctx.asset_crate_id(&file.sub_path_string);
                let asset_crate_url = ctx.crate_url(&asset_crate_id);
                let mut model_crate = ModelCrate::new();
                model_crate
                    .import(&ctx.assets, &file.temp_download_url, true, config.force_assimp, create_texture_resolver(&ctx))
                    .await
                    .with_context(|| format!("Failed to import model {}", file.sub_path_string))?;
                model_crate.model_mut().set_name(&file.sub_path.filename);
                model_crate.create_object();

                config.apply(&ctx, &mut model_crate).await?;

                if config.output_objects {
                    res.push(OutAsset {
                        asset_crate_id: asset_crate_id.clone(),
                        sub_asset: None,
                        type_: AssetType::Object,
                        hidden: false,
                        name: file.sub_path.filename.to_string(),

                        tags: Default::default(),
                        categories: Default::default(),
                        preview: OutAssetPreview::FromModel {
                            url: asset_crate_url.resolve(model_crate.models.loc.path(ModelCrate::MAIN)).unwrap(),
                        },
                        content: OutAssetContent::Content(asset_crate_url.resolve(model_crate.objects.loc.path(ModelCrate::MAIN)).unwrap()),
                        source: Some(file.sub_path_string.clone()),
                    });
                }
                if config.output_animations {
                    for anim in model_crate.animations.content.keys() {
                        res.push(OutAsset {
                            asset_crate_id: asset_crate_id.clone(),
                            sub_asset: Some(format!("anim_{}", slugify::slugify(anim, "", "_", None))),
                            type_: AssetType::Animation,
                            hidden: false,
                            name: file.sub_path.filename.to_string(),
                            tags: Default::default(),
                            categories: Default::default(),
                            preview: OutAssetPreview::None,
                            content: OutAssetContent::Content(asset_crate_url.resolve(model_crate.animations.loc.path(anim)).unwrap()),
                            source: Some(file.sub_path_string.clone()),
                        });
                    }
                }
                ctx.write_model_crate(&model_crate, &asset_crate_id).await;
                Ok(res)
            }
        },
    )
    .await
}
