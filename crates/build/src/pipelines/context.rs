use std::sync::Arc;

use ambient_model_import::model_crate::ModelCrate;
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ModelCrateAssetType, TypedAssetUrl},
};
use anyhow::Context;
use futures::{future::join_all, Future};
use itertools::Itertools;
use relative_path::{RelativePath, RelativePathBuf};
use tokio::sync::Semaphore;

use super::{out_asset::OutAsset, FileCollection, Pipeline, ProcessCtx};

#[derive(Clone)]
pub struct PipelineCtx {
    pub(crate) process_ctx: ProcessCtx,
    pub(crate) files: FileCollection,
    pub(crate) pipeline_file: AbsAssetUrl,
    pub(crate) root_path: RelativePathBuf,

    pub(crate) pipeline: Arc<Pipeline>,
}
impl PipelineCtx {
    pub fn assets(&self) -> &AssetCache {
        &self.process_ctx.assets
    }
    pub fn in_root(&self) -> AbsAssetUrl {
        self.process_ctx
            .in_root
            .push(&self.root_path)
            .unwrap()
            .as_directory()
    }
    pub fn out_root(&self) -> AbsAssetUrl {
        self.process_ctx
            .out_root
            .push(&self.root_path)
            .unwrap()
            .as_directory()
    }
    pub fn pipeline_path(&self) -> RelativePathBuf {
        let path = self
            .process_ctx
            .in_root
            .relative_path(self.pipeline_file.decoded_path());

        if let Some(fragment) = self.pipeline_file.0.fragment() {
            path.join(fragment)
        } else {
            path
        }
    }

    pub async fn write_model_crate(
        &self,
        model_crate: &ModelCrate,
        path: &RelativePath,
    ) -> TypedAssetUrl<ModelCrateAssetType> {
        join_all(
            model_crate
                .to_items()
                .iter()
                .map(|item| self.write_file(path.join(&item.path), (*item.data).clone())),
        )
        .await;
        self.out_root().push(path).unwrap().as_directory().into()
    }
    pub async fn write_file(&self, path: impl AsRef<str>, content: Vec<u8>) -> AbsAssetUrl {
        (self.process_ctx.write_file)(self.root_path.join(path.as_ref()).to_string(), content).await
    }
    pub async fn process_single<F: Future<Output = anyhow::Result<Vec<OutAsset>>> + Send>(
        &self,
        process: impl FnOnce(PipelineCtx) -> F + Sync + Send + 'static,
    ) -> Vec<OutAsset> {
        let res = tokio::spawn({
            let ctx = self.clone();
            async move {
                process(ctx.clone()).await.with_context(|| {
                    format!(
                        "Error while processing single pipeline \"{}\"",
                        ctx.pipeline_path()
                    )
                })
            }
        })
        .await
        .with_context(|| {
            format!(
                "Error while retrieving result of processed single pipeline \"{}\"",
                self.pipeline_path()
            )
        });
        let err = match res {
            Ok(Ok(res)) => return res,
            Ok(Err(err)) => err,
            Err(err) => err,
        };
        (self.process_ctx.on_error)(err).await;
        Vec::new()
    }
    pub async fn process_files<F: Future<Output = anyhow::Result<Vec<OutAsset>>> + Send>(
        &self,
        filter: impl Fn(&AbsAssetUrl) -> bool,
        process_file: impl Fn(PipelineCtx, AbsAssetUrl) -> F + Sync + Send + 'static,
    ) -> Vec<OutAsset> {
        let sources_filter = self
            .pipeline
            .sources
            .iter()
            .map(|p| glob::Pattern::new(p))
            .collect::<Result<Vec<_>, glob::PatternError>>()
            .unwrap();
        log::debug!(
            "[{}] Sources filter: {:?}",
            self.pipeline_path(),
            sources_filter
        );
        let opt_filter = self
            .process_ctx
            .input_file_filter
            .as_ref()
            .and_then(|x| glob::Pattern::new(x).ok());
        log::debug!(
            "[{}] Input file filter: {:?}",
            self.pipeline_path(),
            sources_filter
        );
        let filter_by_sources = move |file: &AbsAssetUrl| {
            if sources_filter.is_empty() {
                true
            } else {
                let path = self.in_root().relative_path(file.decoded_path());
                for pat in &sources_filter {
                    if pat.matches(path.as_str()) {
                        return true;
                    }
                }
                false
            }
        };
        let filter_by_opt_filter = |f: &AbsAssetUrl| {
            let path = self.in_root().relative_path(f.decoded_path());
            opt_filter
                .as_ref()
                .map(|p| p.matches(path.as_str()))
                .unwrap_or(true)
        };
        let files = self
            .files
            .0
            .iter()
            .filter(move |file| {
                if !filter_by_sources(file) {
                    log::debug!(
                        "[{}] Skipping file {} because it doesn't match sources filter",
                        self.pipeline_path(),
                        file
                    );
                    return false;
                }
                if !filter_by_opt_filter(file) {
                    log::debug!(
                        "[{}] Skipping file {} because it doesn't match input file filter",
                        self.pipeline_path(),
                        file
                    );
                    return false;
                }
                if !filter(file) {
                    log::debug!(
                        "[{}] Skipping file {} because it doesn't match pipeline filter",
                        self.pipeline_path(),
                        file
                    );
                    return false;
                }
                true
            })
            .cloned()
            .collect_vec();
        let n_files = files.len();
        let process_file = Arc::new(process_file);
        let semaphore = PipelineFileSemaphore.get(&self.process_ctx.assets);
        join_all(files.into_iter().enumerate().map(move |(i, file)| {
            let ctx = self.clone();
            let process_file = process_file.clone();
            let semaphore = semaphore.clone();
            async move {
                let res = tokio::spawn({
                    let ctx = ctx.clone();
                    let file = file.clone();
                    let file_path = ctx.in_root().relative_path(file.decoded_path());
                    async move {
                        let _permit = semaphore.acquire().await;
                        (ctx.process_ctx.on_status)(format!(
                            "[{}] Processing file {}/{}: {}",
                            ctx.pipeline_path(),
                            i + 1,
                            n_files,
                            file_path
                        ))
                        .await;
                        process_file(ctx.clone(), file.clone())
                            .await
                            .with_context(|| {
                                format!(
                                    "Error while processing pipeline \"{}\" for file \"{file_path}\"",
                                    ctx.pipeline_path()
                                )
                            })
                    }
                })
                .await
                .with_context(|| {
                    format!(
                        "Error while processing pipeline \"{}\" for file \"{}\"",
                        ctx.pipeline_path(),
                        ctx.in_root().relative_path(file.decoded_path())
                    )
                });
                let err = match res {
                    Ok(Ok(res)) => return res,
                    Ok(Err(err)) => err,
                    Err(err) => err,
                };
                (self.process_ctx.on_error)(err).await;
                Vec::new()
            }
        }))
        .await
        .into_iter()
        .flatten()
        .collect()
    }
    pub fn get_downloadable_url(&self, url: &AbsAssetUrl) -> anyhow::Result<&AbsAssetUrl> {
        self.process_ctx
            .files
            .0
            .iter()
            .find(|x| x.decoded_path() == url.decoded_path())
            .with_context(|| format!("Unable to find downloadable URL for `{url}`"))
    }
}

/// Limit the number of concurent file processings to 20
#[derive(Debug)]
struct PipelineFileSemaphore;
impl SyncAssetKey<Arc<Semaphore>> for PipelineFileSemaphore {
    fn load(&self, _assets: AssetCache) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(20))
    }
}
