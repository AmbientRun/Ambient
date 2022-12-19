use std::{borrow::BorrowMut, marker::PhantomData, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use futures::Future;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Semaphore;
use yaml_rust::YamlLoader;

use crate::{
    asset_cache::{AssetCache, AssetKeepalive, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt}, mesh::Mesh
};

pub type UrlString = String;

pub type AssetResult<T> = Result<T, AssetError>;

#[derive(Clone, Error)]
#[error(transparent)]
pub struct AssetError(Arc<anyhow::Error>);

impl From<anyhow::Error> for AssetError {
    fn from(err: anyhow::Error) -> Self {
        Self(Arc::new(err))
    }
}
impl std::fmt::Debug for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertErrorString(pub String);
impl From<AssetError> for AssertErrorString {
    fn from(err: AssetError) -> Self {
        Self(format!("{:#}", err))
    }
}
impl From<anyhow::Error> for AssertErrorString {
    fn from(err: anyhow::Error) -> Self {
        Self(format!("{:#}", err))
    }
}

#[derive(Clone, Debug)]
pub struct AssetsCacheDir;
impl SyncAssetKey<PathBuf> for AssetsCacheDir {
    fn load(&self, _assets: AssetCache) -> PathBuf {
        "tmp".into()
    }
}

#[derive(Clone, Debug)]
pub struct ReqwestClientKey;
impl SyncAssetKey<reqwest::Client> for ReqwestClientKey {
    fn load(&self, _assets: AssetCache) -> reqwest::Client {
        reqwest::Client::new()
    }
}

/// Download with retries and a global rate limiting sempahore
pub async fn download<T, F: Future<Output = anyhow::Result<T>>>(
    assets: &AssetCache,
    url: impl reqwest::IntoUrl,
    map: impl Fn(reqwest::Response) -> F,
) -> anyhow::Result<T> {
    let client = ReqwestClientKey.get(assets);
    let url_str = url.as_str().to_string();
    let url_short = if url_str.len() > 200 { format!("{}...", &url_str[..200]) } else { url_str.to_string() };
    let url: Url = url.into_url()?;

    let max_retries = 12;
    for i in 0..max_retries {
        let semaphore = DownloadSemaphore.get(assets);
        log::info!("download [pending ] {}", url_short);
        let _permit = semaphore.acquire().await.unwrap();
        log::info!("download [download] {}", url_short);
        let resp = client.get(url.clone()).send().await.with_context(|| format!("Failed to download {}", url_str))?;
        if !resp.status().is_success() {
            log::warn!("Request for {} failed: {:?}", url_str, resp.status());
            return Err(anyhow!("Downloading {url_str} failed, bad status code: {:?}", resp.status()));
        }
        match map(resp).await {
            Ok(res) => {
                log::info!("download [complete] {}", url_short);
                return Ok(res);
            }
            Err(err) => {
                log::warn!("Failed to read body of {url_str}, retrying ({i}/{max_retries}): {:?}", err);
                tokio::time::sleep(Duration::from_millis(2u64.pow(i))).await;
            }
        }
    }
    Err(anyhow::anyhow!("Failed to download body of {}", url_str))
}

#[derive(Clone, Debug)]
pub struct BytesFromUrl {
    pub url: UrlString,
    pub cache_on_disk: bool,
}
impl BytesFromUrl {
    pub fn cached(url: impl Into<String>) -> Self {
        Self { url: url.into(), cache_on_disk: true }
    }
    pub fn uncached(url: impl Into<String>) -> Self {
        Self { url: url.into(), cache_on_disk: false }
    }
}
#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<Vec<u8>>>> for BytesFromUrl {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<Vec<u8>>> {
        if self.cache_on_disk {
            let path = BytesFromUrlCachedPath(self.url).get(&assets).await?;
            let semaphore = FileReadSemaphore.get(&assets);
            let _permit = semaphore.acquire().await;
            return Ok(Arc::new(tokio::fs::read(&*path).await.context(format!("Failed to read file: {:?}", path))?));
        }

        let parsed_url = match ContentLoc::parse(&self.url)? {
            ContentLoc::RelativePath(path) => {
                return Ok(Arc::new(tokio::fs::read(&path).await.context(format!("Failed to read file at: {:?}", path))?));
            }
            ContentLoc::Url(url) => url,
        };

        let body = download(&assets, parsed_url.clone(), |resp| async { Ok(resp.bytes().await?) }).await?.to_vec();
        assert!(!body.is_empty());
        Ok(Arc::new(body))
    }
    fn cpu_size(&self, value: &AssetResult<Arc<Vec<u8>>>) -> Option<usize> {
        value.as_ref().ok().map(|v| v.len())
    }
}

/// Get the local cache file location of a resource, and ensure the resource is downloaded to that cache file
#[derive(Clone, Debug)]
pub struct BytesFromUrlCachedPath(pub UrlString);
#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<PathBuf>>> for BytesFromUrlCachedPath {
    fn keepalive(&self) -> AssetKeepalive {
        AssetKeepalive::Forever
    }
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<PathBuf>> {
        use tokio::io::AsyncWriteExt;
        let Self(url) = self;
        let parsed_url = match ContentLoc::parse(&url)? {
            ContentLoc::RelativePath(path) => {
                return Ok(Arc::new(path));
            }
            ContentLoc::Url(url) => url,
        };
        if parsed_url.scheme() == "file" {
            let path = parsed_url.path();
            let path = if let Some(path) = path.strip_prefix("/CWD/") { path } else { path };
            return Ok(Arc::new(path.into()));
        }
        let cache_dir = AssetsCacheDir.try_get(&assets).unwrap_or_else(|| PathBuf::from("tmp"));
        let path = cache_dir.join(&parsed_url.path()[1..]);
        if !path.exists() {
            let mut dir = path.clone();
            dir.pop();
            std::fs::create_dir_all(&dir).context(format!("Failed to create asset dir: {:?}", dir))?;
            let tmp_path = path.with_extension(".downloading");
            download(&assets, parsed_url.clone(), {
                let tmp_path = tmp_path.clone();
                move |mut resp| {
                    let tmp_path = tmp_path.clone();
                    async move {
                        let mut file =
                            tokio::fs::File::create(&tmp_path).await.context(format!("Failed to create file: {:?}", tmp_path))?;
                        while let Some(mut item) = resp.chunk().await.context("Failed to download chunk")? {
                            file.write_all_buf(item.borrow_mut()).await.context("Failed to write to tmp file")?;
                        }
                        file.flush().await.context("Failed to flush tmp file")?;
                        Ok(())
                    }
                }
            })
            .await?;
            std::fs::rename(&tmp_path, &path).context(format!("Failed to rename tmp file, from: {:?}, to: {:?}", tmp_path, path))?;
            log::info!("Cached asset at {:?}", path);
        }
        return Ok(Arc::new(path));
    }
}

/// Limit the number of concurent file reads to 10
#[derive(Debug)]
struct FileReadSemaphore;
impl SyncAssetKey<Arc<Semaphore>> for FileReadSemaphore {
    fn load(&self, _assets: AssetCache) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(10))
    }
}

/// Limit the number of concurent downloads to 5
#[derive(Debug)]
struct DownloadSemaphore;
impl SyncAssetKey<Arc<Semaphore>> for DownloadSemaphore {
    fn load(&self, _assets: AssetCache) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(5))
    }
}

pub enum ContentLoc {
    Url(Url),
    RelativePath(PathBuf),
}
impl ContentLoc {
    pub fn parse(url: &str) -> AssetResult<ContentLoc> {
        if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("file://") {
            Ok(ContentLoc::RelativePath(url.into()))
        } else {
            Ok(Url::parse(url).map(ContentLoc::Url).context(format!("Failed to parse url: {:?}", url))?)
        }
    }
    pub fn ends_with_caseless(&self, end: &str) -> bool {
        match self {
            ContentLoc::Url(url) => url.path().to_lowercase().ends_with(end),
            ContentLoc::RelativePath(path) => path.to_str().unwrap().to_lowercase().ends_with(end),
        }
    }
    pub fn cache_path_buf(&self) -> PathBuf {
        match self {
            ContentLoc::Url(..) => PathBuf::from(self.cache_path_string()),
            ContentLoc::RelativePath(path) => path.clone(),
        }
    }
    pub fn cache_path_string(&self) -> String {
        match self {
            ContentLoc::Url(url) => {
                let path = url.path().to_string();
                if let Some(sub) = path.strip_prefix('/') {
                    sub.to_string()
                } else {
                    path
                }
            }
            ContentLoc::RelativePath(path) => {
                let path = path.as_os_str().to_str().unwrap().to_string();
                if let Some(path) = path.strip_prefix('/') {
                    path.to_string()
                } else {
                    path
                }
            }
        }
    }
}

pub struct JsonFromUrl<T> {
    url: UrlString,
    cache_on_disk: bool,
    _type: PhantomData<T>,
}

impl<T> Clone for JsonFromUrl<T> {
    fn clone(&self) -> Self {
        Self { url: self.url.clone(), cache_on_disk: self.cache_on_disk, _type: self._type }
    }
}

impl<T> JsonFromUrl<T> {
    pub fn cached(url: impl Into<String>) -> Self {
        Self { url: url.into(), cache_on_disk: true, _type: PhantomData }
    }
    pub fn uncached(url: impl Into<String>) -> Self {
        Self { url: url.into(), cache_on_disk: false, _type: PhantomData }
    }
}
impl<T> std::fmt::Debug for JsonFromUrl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadJsonKey").field("url", &self.url).field("_type", &self._type).finish()
    }
}
#[async_trait]
impl<T: DeserializeOwned + Sync + Send + 'static> AsyncAssetKey<AssetResult<Arc<T>>> for JsonFromUrl<T> {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<T>> {
        let data = BytesFromUrl { url: self.url.clone(), cache_on_disk: self.cache_on_disk }.get(&assets).await?;
        Ok(serde_json::from_slice(&data).context("Json failed to parse")?)
    }
}

#[derive(Clone, Debug)]
pub struct YamlFromUrl {
    pub url: UrlString,
    pub cache_on_disk: bool,
}
impl YamlFromUrl {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into(), cache_on_disk: true }
    }
}
#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<Vec<yaml_rust::Yaml>>>> for YamlFromUrl {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<Vec<yaml_rust::Yaml>>> {
        let data = BytesFromUrl { url: self.url.clone(), cache_on_disk: self.cache_on_disk }.get(&assets).await?;
        let data = std::str::from_utf8(&data).context("Bad yaml")?;

        let data =
            data.replace("!u!", "unity_object: ").replacen("unity_object: ", "!u!", 1).replace("--- unity_object: ", "---\nunity_object: ");
        let docs = YamlLoader::load_from_str(&data).context("Bad yaml")?;
        Ok(Arc::new(docs))
    }
}

#[derive(Debug)]
pub struct BincodeFromUrl<T> {
    pub url: UrlString,
    pub cache_on_disk: bool,
    type_: PhantomData<T>,
}

impl<T> Clone for BincodeFromUrl<T> {
    fn clone(&self) -> Self {
        Self { url: self.url.clone(), cache_on_disk: self.cache_on_disk, type_: self.type_ }
    }
}
impl<T> BincodeFromUrl<T> {
    pub fn new(url: impl Into<String>, cache_on_disk: bool) -> Self {
        Self { url: url.into(), cache_on_disk, type_: PhantomData }
    }
    pub fn cached(url: impl Into<String>) -> Self {
        Self::new(url, true)
    }
}
#[async_trait]
impl<T: DeserializeOwned + std::fmt::Debug + Sync + Send + 'static> AsyncAssetKey<AssetResult<Arc<T>>> for BincodeFromUrl<T> {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<T>> {
        let data = BytesFromUrl { url: self.url.clone(), cache_on_disk: self.cache_on_disk }.get(&assets).await?;
        Ok(Arc::new(bincode::deserialize(&data).context("Failed to deserialize")?))
    }
}

pub type MeshFromUrl = BincodeFromUrl<Mesh>;
