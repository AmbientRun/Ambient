use std::{marker::PhantomData, path::PathBuf, sync::Arc, time::Duration};

use crate::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    mesh::Mesh,
};
use ambient_sys::task::wasm_nonsend;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use futures::Future;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use tokio::sync::Semaphore;

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
        Self(format!("{err:#}"))
    }
}
impl From<anyhow::Error> for AssertErrorString {
    fn from(err: anyhow::Error) -> Self {
        Self(format!("{err:#}"))
    }
}

#[derive(Clone, Debug)]
pub struct AssetsCacheDir;
impl SyncAssetKey<PathBuf> for AssetsCacheDir {
    fn load(&self, _assets: AssetCache) -> PathBuf {
        std::env::current_dir().unwrap().join("tmp")
    }
}

#[derive(Clone, Debug)]
pub struct AssetsCacheOnDisk;
impl SyncAssetKey<bool> for AssetsCacheOnDisk {
    fn load(&self, _assets: AssetCache) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct ReqwestClientKey;
impl SyncAssetKey<reqwest::Client> for ReqwestClientKey {
    fn load(&self, _assets: AssetCache) -> reqwest::Client {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "User-Agent",
            reqwest::header::HeaderValue::from_static(concat!(
                "Ambient/",
                env!("CARGO_PKG_VERSION")
            )),
        );
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    }
}

/// Download with retries and a global rate limiting sempahore
pub(crate) async fn download<T: 'static + Send, F: Future<Output = anyhow::Result<T>>>(
    assets: &AssetCache,
    url: impl reqwest::IntoUrl,
    map: impl 'static + Send + Fn(reqwest::Response) -> F,
) -> anyhow::Result<T> {
    let url_str = url.as_str().to_string();
    let url = url.into_url()?;
    let assets = assets.clone();

    // reqwest::Client is not Send on wasm
    wasm_nonsend(move || async move {
        let client = ReqwestClientKey.get(&assets);
        let url_short = if url_str.len() > 200 {
            format!("{}...", &url_str[..200])
        } else {
            url_str.to_string()
        };

        let max_retries = 12;
        for i in 0..max_retries {
            let semaphore = DownloadSemaphore.get(&assets);
            log::info!("download [pending ] {}", url_short);
            let _permit = semaphore.acquire().await.unwrap();
            log::info!("download [download] {}", url_short);
            let resp = client
                .get(url.clone())
                .send()
                .await
                .with_context(|| format!("Failed to download {url_str}"))?;
            if !resp.status().is_success() {
                log::warn!("Request for {} failed: {:?}", url_str, resp.status());
                return Err(anyhow!(
                    "Downloading {url_str} failed, bad status code: {:?}",
                    resp.status()
                ));
            }
            match map(resp).await {
                Ok(res) => {
                    log::info!("download [complete] {}", url_short);
                    return Ok(res);
                }
                Err(err) => {
                    log::warn!(
                        "Failed to read body of {url_str}, retrying ({i}/{max_retries}): {:?}",
                        err
                    );
                    ambient_sys::time::sleep(Duration::from_millis(2u64.pow(i))).await;
                }
            }
        }

        Err(anyhow::anyhow!("Failed to download body of {}", url_str))
    })
    .await
}

#[derive(Clone, Debug)]
pub struct BytesFromUrl {
    pub url: AbsAssetUrl,
    pub cache_on_disk: bool,
}

impl BytesFromUrl {
    pub fn new(url: AbsAssetUrl, cache_on_disk: bool) -> Self {
        Self { url, cache_on_disk }
    }
    pub fn parse_url(url: impl AsRef<str>, cache_on_disk: bool) -> anyhow::Result<Self> {
        Ok(Self {
            url: AbsAssetUrl::from_str(url.as_ref())?,
            cache_on_disk,
        })
    }
}

/// Use [BytesFromUrl] unless you _really_ need uncached downloads
pub async fn download_uncached_bytes(
    assets: &AssetCache,
    url: AbsAssetUrl,
) -> AssetResult<Vec<u8>> {
    if let Some(path) = url.to_file_path()? {
        return Ok(ambient_sys::fs::read(path)
            .await
            .context(format!("Failed to read file at: {:}", url.0))?);
    }

    let body = download(
        assets,
        url.to_download_url(assets).map_err(anyhow::Error::new)?.0,
        |resp| async { Ok(resp.bytes().await?) },
    )
    .await?
    .to_vec();
    assert!(!body.is_empty());
    Ok(body)
}

#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<Vec<u8>>>> for BytesFromUrl {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<Vec<u8>>> {
        #[cfg(not(target_os = "unknown"))]
        if self.cache_on_disk && AssetsCacheOnDisk.get(&assets) {
            let path = BytesFromUrlCachedPath {
                url: self.url.clone(),
            }
            .get(&assets)
            .await?;
            let semaphore = FileReadSemaphore.get(&assets);
            let _permit = semaphore.acquire().await;
            return Ok(Arc::new(
                ambient_sys::fs::read(&*path)
                    .await
                    .context(format!("Failed to read file: {path:?}"))?,
            ));
        }

        download_uncached_bytes(&assets, self.url.clone())
            .await
            .map(Arc::new)
    }

    fn cpu_size(&self, value: &AssetResult<Arc<Vec<u8>>>) -> Option<u64> {
        // NOTE: on wasm bytes is limited to 4gb
        value.as_ref().ok().map(|v| v.len() as u64)
    }
}

/// Get the local cache file location of a resource, and ensure the resource is downloaded to that cache file
#[derive(Clone, Debug)]
#[cfg(not(target_os = "unknown"))]
pub struct BytesFromUrlCachedPath {
    pub url: AbsAssetUrl,
}

#[cfg(not(target_os = "unknown"))]
impl BytesFromUrlCachedPath {
    pub fn parse_url(url: impl AsRef<str>) -> anyhow::Result<Self> {
        Ok(Self {
            url: AbsAssetUrl::from_str(url.as_ref())?,
        })
    }
}

#[async_trait]
#[cfg(not(target_os = "unknown"))]
impl AsyncAssetKey<AssetResult<Arc<PathBuf>>> for BytesFromUrlCachedPath {
    fn keepalive(&self) -> ambient_asset_cache::AssetKeepalive {
        ambient_asset_cache::AssetKeepalive::Forever
    }
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<PathBuf>> {
        if let Some(path) = self.url.to_file_path()? {
            return Ok(Arc::new(path));
        }

        let path = self.url.absolute_cache_path(&assets);
        if !path.exists() {
            use tokio::io::AsyncWriteExt;
            let mut dir = path.clone();
            dir.pop();
            std::fs::create_dir_all(&dir)
                .context(format!("Failed to create asset dir: {dir:?}"))?;
            let tmp_path = path.with_extension(".downloading");
            download(
                &assets,
                self.url
                    .to_download_url(&assets)
                    .map_err(anyhow::Error::new)?
                    .0,
                {
                    let tmp_path = tmp_path.clone();
                    move |mut resp| {
                        let tmp_path = tmp_path.clone();
                        async move {
                            let mut file = tokio::fs::File::create(&tmp_path)
                                .await
                                .context(format!("Failed to create file: {tmp_path:?}"))?;
                            use std::borrow::BorrowMut;
                            while let Some(mut item) =
                                resp.chunk().await.context("Failed to download chunk")?
                            {
                                file.write_all_buf(item.borrow_mut())
                                    .await
                                    .context("Failed to write to tmp file")?;
                            }
                            file.flush().await.context("Failed to flush tmp file")?;
                            Ok(())
                        }
                    }
                },
            )
            .await?;
            std::fs::rename(&tmp_path, &path).context(format!(
                "Failed to rename tmp file, from: {tmp_path:?}, to: {path:?}"
            ))?;
            log::info!("Cached asset at {:?}", path);
        }

        return Ok(Arc::new(path));
    }
}

/// Limit the number of conccurent file reads to 10
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

pub struct JsonFromUrl<T> {
    url: AbsAssetUrl,
    cache_on_disk: bool,
    _type: PhantomData<T>,
}

impl<T> Clone for JsonFromUrl<T> {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            cache_on_disk: self.cache_on_disk,
            _type: self._type,
        }
    }
}

impl<T> JsonFromUrl<T> {
    pub fn new(url: AbsAssetUrl, cache_on_disk: bool) -> Self {
        Self {
            url,
            cache_on_disk,
            _type: PhantomData,
        }
    }
    pub fn parse_url(url: impl AsRef<str>, cache_on_disk: bool) -> anyhow::Result<Self> {
        Ok(Self {
            url: AbsAssetUrl::from_str(url.as_ref())?,
            cache_on_disk,
            _type: PhantomData,
        })
    }
}
impl<T> std::fmt::Debug for JsonFromUrl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadJsonKey")
            .field("url", &self.url)
            .field("_type", &self._type)
            .finish()
    }
}
#[async_trait]
impl<T: DeserializeOwned + Sync + Send + 'static> AsyncAssetKey<AssetResult<Arc<T>>>
    for JsonFromUrl<T>
{
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<T>> {
        let data = BytesFromUrl {
            url: self.url.clone(),
            cache_on_disk: self.cache_on_disk,
        }
        .get(&assets)
        .await?;
        Ok(serde_json::from_slice(&data).context("Json failed to parse")?)
    }
}

#[derive(Debug)]
pub struct BincodeFromUrl<T> {
    pub url: AbsAssetUrl,
    pub cache_on_disk: bool,
    type_: PhantomData<T>,
}

impl<T> Clone for BincodeFromUrl<T> {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            cache_on_disk: self.cache_on_disk,
            type_: self.type_,
        }
    }
}
impl<T> BincodeFromUrl<T> {
    pub fn new(url: AbsAssetUrl, cache_on_disk: bool) -> Self {
        Self {
            url,
            cache_on_disk,
            type_: PhantomData,
        }
    }
    pub fn parse_url(url: impl AsRef<str>, cache_on_disk: bool) -> anyhow::Result<Self> {
        Ok(Self {
            url: AbsAssetUrl::from_str(url.as_ref())?,
            cache_on_disk,
            type_: PhantomData,
        })
    }
}
#[async_trait]
impl<T: DeserializeOwned + std::fmt::Debug + Sync + Send + 'static>
    AsyncAssetKey<AssetResult<Arc<T>>> for BincodeFromUrl<T>
{
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<T>> {
        let data = BytesFromUrl {
            url: self.url.clone(),
            cache_on_disk: self.cache_on_disk,
        }
        .get(&assets)
        .await?;
        Ok(Arc::new(
            bincode::deserialize(&data).context("Failed to deserialize")?,
        ))
    }
}

pub type MeshFromUrl = BincodeFromUrl<Mesh>;
