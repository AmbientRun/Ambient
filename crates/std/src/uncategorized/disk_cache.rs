use std::{fmt::Debug, path::PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    asset_cache::{Asset, AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt},
    download_asset::{AssetError, AssetsCacheDir},
    sha256_digest,
};

/// This can wrap any resource which is Serializable, and will cache that resource to disk
/// for faster loads in the future
#[derive(Debug, Clone)]
pub struct DiskCachedJson<T>(pub T);

#[async_trait]
impl<
        V: Serialize + DeserializeOwned + Sync + Send + Clone + Asset + 'static,
        T: Debug + Clone + Sync + Send + AsyncAssetKeyExt<Result<V, AssetError>>,
    > AsyncAssetKey<Result<V, AssetError>> for DiskCachedJson<T>
{
    async fn load(self, assets: AssetCache) -> Result<V, AssetError> {
        let cache_dir = AssetsCacheDir
            .try_get(&assets)
            .unwrap_or_else(|| PathBuf::from("tmp"));
        let cache_dir = cache_dir.join("json_cache");
        std::fs::create_dir_all(&cache_dir).context("Failed to created cache dir")?;
        let cache_key = sha256_digest(&format!("{:?}", self.0));
        let file = cache_dir.join(cache_key);
        if file.exists() {
            let data = ambient_sys::fs::read(&file)
                .await
                .context("Failed to read cache file")?;
            match serde_json::from_slice(&data) {
                Ok(value) => return Ok(value),
                Err(err) => {
                    log::warn!("Failed to parse cached json asset: {:?}", err);
                }
            }
        }
        let value = self.0.get(&assets).await?;
        ambient_sys::fs::write(file, serde_json::to_string(&value).unwrap())
            .await
            .context("Failed to write cache file")?;
        Ok(value)
    }
}
