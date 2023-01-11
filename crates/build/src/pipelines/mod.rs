use std::{collections::HashSet, sync::Arc};

use context::{AssetCrate, PipelineCtx};
use elements_model_import::model_crate::ModelCrate;
use elements_std::{asset_cache::AssetCache, asset_url::AssetType};
use futures::future::join_all;
use itertools::Itertools;
use out_asset::{OutAsset, OutAssetContent, OutAssetPreview};
use serde::{Deserialize, Serialize};

use self::{materials::MaterialsPipeline, models::ModelsPipeline};
use crate::helpers::download_bytes;

pub mod audio;
pub mod context;
pub mod materials;
pub mod models;
pub mod out_asset;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipelineConfig {
    Models(ModelsPipeline),
    ScriptBundles,
    Materials(MaterialsPipeline),
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct Pipeline {
    pub pipeline: PipelineConfig,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<Vec<String>>,
}
impl Pipeline {
    pub async fn process(&self, ctx: PipelineCtx) -> Vec<OutAsset> {
        let mut assets = match &self.pipeline {
            PipelineConfig::Models(config) => models::pipeline(&ctx, config.clone()).await,
            PipelineConfig::ScriptBundles => {
                ctx.process_files(
                    |f| f.sub_path.extension == Some("script_bundle".to_string()),
                    |ctx, file| async move {
                        let id = ctx.asset_crate_id(&file.sub_path_string);
                        let asset_crate = AssetCrate::new(&ctx, id.clone());
                        let bundle = download_bytes(&ctx.assets, &file.temp_download_url).await.unwrap();
                        let content =
                            asset_crate.write_file(AssetType::ScriptBundle, &format!("{}.script_bundle", ModelCrate::MAIN), bundle).await;

                        Ok(vec![OutAsset {
                            asset_crate_id: id,
                            sub_asset: None,
                            type_: AssetType::ScriptBundle,
                            hidden: false,
                            name: file.sub_path.filename.to_string(),
                            tags: Vec::new(),
                            categories: Default::default(),
                            preview: OutAssetPreview::None,
                            content: OutAssetContent::Content(content),
                            source: Some(file.sub_path_string.clone()),
                        }])
                    },
                )
                .await
            }
            PipelineConfig::Materials(config) => materials::pipeline(&ctx, config.clone()).await,
            PipelineConfig::Audio => audio::pipeline(&ctx).await,
        };
        let ctx = &ctx;
        for asset in &mut assets {
            asset.tags.extend(self.tags.clone());
            for i in 0..asset.categories.len() {
                if let Some(cat) = self.categories.get(i) {
                    asset.categories[i].extend(cat.iter().cloned().collect::<HashSet<_>>());
                }
            }
        }
        assets
    }
}
