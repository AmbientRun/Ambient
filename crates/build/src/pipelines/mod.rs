use std::{collections::HashSet, sync::Arc};

use ambient_asset_cache::SyncAssetKey;
use ambient_pipeline_types::{
    models::ModelsPipeline,
    pipeline::{Pipeline, PipelineProcessor},
};
use ambient_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};
use anyhow::Context;
use context::PipelineCtx;
use futures::{
    future::{ready, BoxFuture},
    stream, Stream, StreamExt, TryStreamExt,
};
use image::ImageFormat;
use out_asset::{OutAsset, OutAssetContent, OutAssetPreview};
use serde::{Deserialize, Serialize};

pub mod audio;
pub mod context;
pub mod materials;
pub mod models;
pub mod out_asset;

pub async fn process_pipeline(pipeline: &Pipeline, ctx: PipelineCtx) -> Vec<OutAsset> {
    let mut assets = match &pipeline.processor {
        PipelineProcessor::Models(config) => models::pipeline(&ctx, config.clone()).await,
        PipelineProcessor::Materials(config) => materials::pipeline(&ctx, config.clone()).await,
        PipelineProcessor::Audio(config) => audio::pipeline(&ctx, config.clone()).await,
    };

    for asset in &mut assets {
        asset.tags.extend(pipeline.tags.clone());
        for i in 0..asset.categories.len() {
            if let Some(cat) = pipeline.categories.get(i) {
                asset.categories[i].extend(cat.iter().cloned().collect::<HashSet<_>>());
            }
        }
    }

    assets
}

/// The outermost structure of the pipeline.toml file.
///
/// Is a struct of arrays of pipelines as toml does not support top-level arrays
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PipelineSchema {
    pub(crate) pipelines: Vec<Pipeline>,
}

fn get_pipelines(
    ctx: &ProcessCtx,
) -> impl Stream<Item = anyhow::Result<(&AbsAssetUrl, PipelineSchema)>> {
    stream::iter(ctx.files.0.iter())
        .filter(|file| ready(file.decoded_path().ends_with("pipeline.toml")))
        .then(move |file| async move {
            let schema = file
                .download_toml::<PipelineSchema>(&ctx.assets)
                .await
                .with_context(|| format!("Failed to read pipeline {:?}", file.0.path()))?;

            Ok((file, schema))
        })
}

pub async fn process_pipelines(ctx: &ProcessCtx) -> anyhow::Result<Vec<OutAsset>> {
    tracing::info!(?ctx.out_root, "Processing pipeline");

    get_pipelines(ctx)
        .map_ok(|(file, pipelines)| {
            futures::stream::iter(pipelines.pipelines.into_iter().enumerate().map(
                |(i, pipeline)| {
                    let mut file = file.clone();
                    file.0.set_fragment(Some(&i.to_string()));
                    Ok((file, pipeline)) as anyhow::Result<_>
                },
            ))
        })
        .try_flatten()
        .map_ok(|(pipeline_file, pipeline): (AbsAssetUrl, Pipeline)| {
            let root = pipeline_file.join(".").unwrap();
            let ctx = PipelineCtx {
                files: ctx.files.sub_directory(root.decoded_path().as_str()),
                process_ctx: ctx.clone(),
                pipeline: Arc::new(pipeline.clone()),
                pipeline_file,
                root_path: ctx.in_root.relative_path(root.decoded_path()),
            };

            async move {
                tokio::spawn(async move { process_pipeline(&pipeline, ctx).await })
                    .await
                    .context("Pipeline processing panicked")
            }
        })
        .try_buffered(30)
        .map_ok(|out_assets| futures::stream::iter(out_assets.into_iter().map(Ok)))
        .try_flatten()
        .try_collect::<Vec<_>>()
        .await
}

#[derive(Debug, Clone)]
pub struct ProcessCtxKey;
impl SyncAssetKey<ProcessCtx> for ProcessCtxKey {}

#[derive(Clone)]
pub struct ProcessCtx {
    pub(crate) assets: AssetCache,
    pub(crate) files: FileCollection,
    pub(crate) input_file_filter: Option<String>,
    pub(crate) package_name: String,
    pub(crate) in_root: AbsAssetUrl,
    pub(crate) out_root: AbsAssetUrl,
    pub(crate) write_file:
        Arc<dyn Fn(String, Vec<u8>) -> BoxFuture<'static, AbsAssetUrl> + Sync + Send>,
    pub(crate) on_status: Arc<dyn Fn(String) -> BoxFuture<'static, ()> + Sync + Send>,
    pub(crate) on_error: Arc<dyn Fn(anyhow::Error) -> BoxFuture<'static, ()> + Sync + Send>,
}

impl std::fmt::Debug for ProcessCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessCtx")
            .field("assets", &self.assets)
            .field("files", &self.files)
            .field("input_file_filter", &self.input_file_filter)
            .field("package_name", &self.package_name)
            .field("in_root", &self.in_root)
            .field("out_root", &self.out_root)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FileCollection(pub Arc<Vec<AbsAssetUrl>>);
impl FileCollection {
    pub fn has_input_file(&self, url: &AbsAssetUrl) -> bool {
        self.0.iter().any(|x| x == url)
    }
    pub fn find_file_res(&self, glob_pattern: impl AsRef<str>) -> anyhow::Result<&AbsAssetUrl> {
        self.find_file(&glob_pattern)
            .with_context(|| format!("Failed to find file with pattern {}", glob_pattern.as_ref()))
    }
    pub fn find_file(&self, glob_pattern: impl AsRef<str>) -> Option<&AbsAssetUrl> {
        let pattern = glob::Pattern::new(glob_pattern.as_ref()).unwrap();
        self.0
            .iter()
            .find(|f| pattern.matches(f.decoded_path().as_str()))
    }
    pub fn sub_directory(&self, path: &str) -> Self {
        Self(Arc::new(
            self.0
                .iter()
                .filter(|url| url.decoded_path().starts_with(path))
                .cloned()
                .collect(),
        ))
    }
}

pub async fn download_image(
    assets: &AssetCache,
    url: &AbsAssetUrl,
) -> anyhow::Result<image::DynamicImage> {
    let data = url.download_bytes(assets).await?;
    if let Some(format) = url
        .extension()
        .as_ref()
        .and_then(ImageFormat::from_extension)
    {
        Ok(image::load_from_memory_with_format(&data, format)
            .with_context(|| format!("Failed to load image {url}"))?)
    } else {
        Ok(
            image::load_from_memory(&data)
                .with_context(|| format!("Failed to load image {url}"))?,
        )
    }
}
