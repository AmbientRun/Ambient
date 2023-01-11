use std::{collections::HashMap, fmt::Display, sync::Arc, time::Duration};

use anyhow::Context;
use elements_model_import::model_crate::ModelCrate;
use elements_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt}, asset_url::{AbsAssetUrl, AssetType}
};
use futures::{
    future::{join_all, BoxFuture}, stream::StreamExt, Future
};
use itertools::Itertools;
use relative_path::RelativePathBuf;
use serde::Serialize;
use tokio::sync::Semaphore;

use super::out_asset::OutAsset;

#[derive(Debug, Clone)]
pub struct AssetCrateId {
    pack_id: String,
    pub crate_uid: String,
}
impl AssetCrateId {
    pub fn new(pack_id: impl Into<String>, crate_uid: &str) -> Self {
        AssetCrateId { pack_id: pack_id.into(), crate_uid: slugify::slugify(crate_uid, "", "_", None) }
    }
}
impl Display for AssetCrateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.pack_id, self.crate_uid)
    }
}

#[derive(Clone)]
pub struct PipelineCtx {
    pub pipeline_run_id: String,
    pub assets: AssetCache,
    pub asset_pack_id: String,
    pub asset_pack_version: String,
    pub asset_pack_name: String,
    pub files: Arc<HashMap<String, AssetPackFile>>,
    pub sources: Vec<String>,
    pub input_file_filter: Option<String>,
    pub pipeline_name: String,
    pub output_base_url: AbsAssetUrl,

    pub write_file: Arc<dyn Fn(String, Vec<u8>) -> BoxFuture<'static, ()> + Sync + Send>,
    pub on_status: Arc<dyn Fn(String) -> BoxFuture<'static, ()> + Sync + Send>,
    pub on_error: Arc<dyn Fn(anyhow::Error) -> BoxFuture<'static, ()> + Sync + Send>,
}
impl PipelineCtx {
    pub fn asset_crate_id(&self, uid: &str) -> AssetCrateId {
        // This does not include the version, because the idea here is that the id's are stable across versions
        // This makes it possible to create collections of assets, that continue to work even if the assets are updated
        AssetCrateId::new(&self.asset_pack_id, uid)
    }

    /// Generate a public url to a file in an asset crate generated from this pack (via the api/v1/assetdb http interface)
    pub fn crate_url(&self, id: &AssetCrateId) -> AbsAssetUrl {
        self.output_base_url.join(self.crate_storage_path(id)).unwrap()
    }
    pub fn pipeline_storage_path(&self) -> String {
        format!("crates/{}/{}", self.asset_pack_id, self.asset_pack_version)
    }
    pub fn crate_storage_path(&self, id: &AssetCrateId) -> String {
        format!("{}/{}", self.pipeline_storage_path(), id.crate_uid)
    }

    pub async fn write_model_crate(&self, model_crate: &ModelCrate, id: &AssetCrateId) {
        let items = model_crate.to_items();
        join_all(items.into_iter().map(|item| self.write_file(format!("{}/{}", id.crate_uid, item.path), (*item.data).clone()))).await;
    }
    pub async fn write_file(&self, path: String, content: Vec<u8>) {
        let path = format!("{}/{}", self.pipeline_storage_path(), path);
        (self.write_file)(path, content).await
    }
    pub async fn process_single<F: Future<Output = anyhow::Result<Vec<OutAsset>>> + Send>(
        &self,
        process: impl FnOnce(PipelineCtx) -> F + Sync + Send + 'static,
    ) -> Vec<OutAsset> {
        let res = tokio::spawn({
            let ctx = self.clone();
            async move { process(ctx.clone()).await.with_context(|| format!("In pipeline {}", ctx.pipeline_name)) }
        })
        .await
        .with_context(|| format!("In pipeline {}", self.pipeline_name));
        let err = match res {
            Ok(Ok(res)) => return res,
            Ok(Err(err)) => err,
            Err(err) => err,
        };
        (self.on_error)(err).await;
        Vec::new()
    }
    pub async fn process_files<F: Future<Output = anyhow::Result<Vec<OutAsset>>> + Send>(
        &self,
        filter: impl Fn(&AssetPackFile) -> bool,
        process_file: impl Fn(PipelineCtx, AssetPackFile) -> F + Sync + Send + 'static,
    ) -> Vec<OutAsset> {
        let sources_filter = self.sources.iter().map(|p| glob::Pattern::new(p)).collect::<Result<Vec<_>, glob::PatternError>>().unwrap();
        let opt_filter = self.input_file_filter.as_ref().and_then(|x| glob::Pattern::new(x).ok());
        let files = self
            .files
            .values()
            .filter(move |file| {
                if sources_filter.is_empty() {
                    true
                } else {
                    for pat in &sources_filter {
                        if pat.matches(&file.sub_path_string) {
                            return true;
                        }
                    }
                    false
                }
            })
            .filter(|f| opt_filter.as_ref().map(|p| p.matches(&f.sub_path_string)).unwrap_or(true))
            .filter(|f| filter(f))
            .cloned()
            .collect_vec();
        let n_files = files.len();
        let process_file = Arc::new(process_file);
        let semaphore = PipelineFileSemaphore.get(&self.assets);
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
                        (ctx.on_status)(format!(
                            "[{}] Processing file {}/{}: {:?}",
                            ctx.pipeline_name,
                            i + 1,
                            n_files,
                            file.sub_path_string
                        ))
                        .await;
                        process_file(ctx.clone(), file.clone())
                            .await
                            .with_context(|| format!("In pipeline {}, at file {}", ctx.pipeline_name, file.sub_path_string))
                    }
                })
                .await
                .with_context(|| format!("In pipeline {}, at file {}", ctx.pipeline_name, file.sub_path_string));
                let err = match res {
                    Ok(Ok(res)) => return res,
                    Ok(Err(err)) => err,
                    Err(err) => err,
                };
                (self.on_error)(err).await;
                Vec::new()
            }
        }))
        .await
        .into_iter()
        .flatten()
        .collect()
    }
    pub fn get_file(&self, path: &str) -> anyhow::Result<&AssetPackFile> {
        self.files.get(path).with_context(|| format!("No such file: {path}"))
    }
}

#[derive(Clone)]
pub struct AssetCrate {
    id: AssetCrateId,
    ctx: PipelineCtx,
}
impl AssetCrate {
    pub fn new(ctx: &PipelineCtx, id: AssetCrateId) -> Self {
        Self { id, ctx: ctx.clone() }
    }
    pub fn get_path(asset_type: AssetType, filename: &str) -> RelativePathBuf {
        format!("{}s/{}", asset_type.to_snake_case(), filename).into()
    }
    pub fn get_public_url(&self, asset_type: AssetType, filename: &str) -> AbsAssetUrl {
        AbsAssetUrl::parse(format!("{}/{}", self.ctx.crate_url(&self.id), Self::get_path(asset_type, filename))).unwrap()
    }
    pub async fn write_file(&self, asset_type: AssetType, filename: &str, content: Vec<u8>) -> AbsAssetUrl {
        let full_path = format!("{}/{}", self.id.crate_uid, Self::get_path(asset_type, filename));
        self.ctx.write_file(full_path, content).await;
        self.get_public_url(asset_type, filename)
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

#[derive(Debug, Clone)]
pub struct AssetPackFile {
    pub sub_path: UrlPath,
    pub sub_path_string: String,
    pub temp_download_url: AbsAssetUrl,
}

#[derive(Debug, Clone)]
pub struct UrlPath {
    pub path: Vec<String>,
    pub filename: String,
    pub extension: Option<String>,
}
impl UrlPath {
    pub fn from_string(value: &str) -> Option<Self> {
        let mut parts = value.split('/').collect_vec();
        let file = parts.pop()?;
        let path = parts.into_iter().map(|x| x.to_string()).collect_vec();
        if let Some((filename, extension)) = file.rsplit_once('.') {
            Some(Self { path, filename: filename.to_string(), extension: Some(extension.to_lowercase()) })
        } else {
            Some(Self { path, filename: file.to_string(), extension: None })
        }
    }
}
#[test]
fn test_url_path() {
    let path = UrlPath::from_string("packs/Y3FxdwvLWHBtJMsQTNbt/1.0.0/180 Turn W_ Briefcase (1).fbx").unwrap();
    assert_eq!(&path.filename, "180 Turn W_ Briefcase (1)");
}
