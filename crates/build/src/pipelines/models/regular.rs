use ambient_model_import::{model_crate::ModelCrate, MODEL_EXTENSIONS};
use ambient_native_std::asset_url::AssetType;
use anyhow::Context;

use super::{
    super::{
        context::PipelineCtx,
        out_asset::{OutAssetContent, OutAssetPreview},
        ModelsPipeline,
    },
    apply, create_texture_resolver,
};
use crate::pipelines::{out_asset::asset_id_from_url, OutAsset};

pub async fn pipeline(ctx: &PipelineCtx, config: ModelsPipeline) -> Vec<OutAsset> {
    ctx.process_files(
        |f| {
            MODEL_EXTENSIONS
                .iter()
                .any(|x| x == &f.extension().unwrap_or_default().to_lowercase())
        },
        move |ctx, file| {
            let config = config.clone();
            async move {
                let mut res = Vec::new();

                let mut model_crate = ModelCrate::new();
                model_crate
                    .import(
                        &ctx.process_ctx.assets,
                        &file,
                        true,
                        config.force_assimp,
                        create_texture_resolver(&ctx),
                    )
                    .await
                    .with_context(|| format!("Failed to import model \"{file}\""))?;
                model_crate
                    .model_mut()
                    .set_name(file.decoded_path().file_name().unwrap());
                model_crate.create_prefab_from_model();

                let out_model_path = ctx.in_root().relative_path(file.decoded_path());
                apply(&config, &ctx, &mut model_crate, &out_model_path).await?;

                let model_crate_url = ctx.write_model_crate(&model_crate, &out_model_path).await;

                if config.output_prefabs {
                    res.push(OutAsset {
                        id: asset_id_from_url(&file),
                        type_: AssetType::Prefab,
                        hidden: false,
                        name: file.decoded_path().file_name().unwrap().to_string(),

                        tags: Default::default(),
                        categories: Default::default(),
                        preview: OutAssetPreview::FromModel {
                            url: model_crate_url.model().abs().unwrap(),
                        },
                        content: OutAssetContent::Content(model_crate_url.prefab().abs().unwrap()),
                        source: Some(file.clone()),
                    });
                }
                if config.output_animations {
                    for anim in model_crate.animations.content.keys() {
                        res.push(OutAsset {
                            id: asset_id_from_url(&file.push(anim).unwrap()),
                            type_: AssetType::Animation,
                            hidden: false,
                            name: file.decoded_path().file_name().unwrap().to_string(),
                            tags: Default::default(),
                            categories: Default::default(),
                            preview: OutAssetPreview::None,
                            content: OutAssetContent::Content(
                                model_crate_url.animation(anim).abs().unwrap(),
                            ),
                            source: Some(file.clone()),
                        });
                    }
                }
                Ok(res)
            }
        },
    )
    .await
}
