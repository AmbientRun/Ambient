use std::sync::Arc;

use anyhow::Context;
use elements_model_import::model_crate::ModelCrate;
use elements_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt}, asset_url::{AbsAssetUrl, ModelCrateAssetType, TypedAssetUrl}
};
use futures::{future::join_all, Future};
use itertools::Itertools;
use relative_path::{RelativePath, RelativePathBuf};
use tokio::sync::Semaphore;

use super::{out_asset::OutAsset, Pipeline, ProcessCtx};

#[derive(Clone)]
pub struct PipelineCtx {
    pub process_ctx: ProcessCtx,
    pub pipeline_file: AbsAssetUrl,
    pub root_path: RelativePathBuf,

    pub pipeline: Arc<Pipeline>,
}
impl PipelineCtx {
    pub fn assets(&self) -> &AssetCache {
        &self.process_ctx.assets
    }
    pub fn in_root(&self) -> AbsAssetUrl {
        self.process_ctx.in_root.push(&self.root_path).unwrap()
    }
    pub fn out_root(&self) -> AbsAssetUrl {
        self.process_ctx.out_root.push(&self.root_path).unwrap()
    }

    pub async fn write_model_crate(&self, model_crate: &ModelCrate, path: &RelativePath) -> TypedAssetUrl<ModelCrateAssetType> {
        join_all(model_crate.to_items().iter().map(|item| self.write_file(path.join(&item.path), (*item.data).clone()))).await;
        self.out_root().push(path).unwrap().into()
    }
    pub async fn write_file(&self, path: impl AsRef<str>, content: Vec<u8>) -> AbsAssetUrl {
        (self.process_ctx.write_file)(path.as_ref().to_string(), content).await
    }
    pub async fn process_single<F: Future<Output = anyhow::Result<Vec<OutAsset>>> + Send>(
        &self,
        process: impl FnOnce(PipelineCtx) -> F + Sync + Send + 'static,
    ) -> Vec<OutAsset> {
        let res = tokio::spawn({
            let ctx = self.clone();
            async move { process(ctx.clone()).await.with_context(|| format!("In pipeline {}", ctx.pipeline_file)) }
        })
        .await
        .with_context(|| format!("In pipeline {}", self.pipeline_file));
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
        let sources_filter =
            self.pipeline.sources.iter().map(|p| glob::Pattern::new(p)).collect::<Result<Vec<_>, glob::PatternError>>().unwrap();
        let opt_filter = self.process_ctx.input_file_filter.as_ref().and_then(|x| glob::Pattern::new(x).ok());
        let files = self
            .process_ctx
            .files
            .iter()
            .filter(move |file| {
                if sources_filter.is_empty() {
                    true
                } else {
                    let path = self.in_root().relative_path(file.path());
                    for pat in &sources_filter {
                        if pat.matches(path.as_str()) {
                            return true;
                        }
                    }
                    false
                }
            })
            .filter(|f| {
                let path = self.in_root().relative_path(f.path());
                opt_filter.as_ref().map(|p| p.matches(path.as_str())).unwrap_or(true)
            })
            .filter(|f| filter(f))
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
                    async move {
                        let _permit = semaphore.acquire().await;
                        (ctx.process_ctx.on_status)(format!("[{}] Processing file {}/{}: {}", ctx.pipeline_file, i + 1, n_files, file))
                            .await;
                        process_file(ctx.clone(), file.clone())
                            .await
                            .with_context(|| format!("In pipeline {}, at file {}", ctx.pipeline_file, file))
                    }
                })
                .await
                .with_context(|| format!("In pipeline {}, at file {}", ctx.pipeline_file, file));
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
        self.process_ctx.files.iter().find(|x| x.path() == url.path()).with_context(|| format!("No such file: {url}"))
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
