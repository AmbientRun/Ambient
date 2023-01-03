use std::sync::Arc;

use async_trait::*;
use elements_std::{
    asset_cache::{AsyncAssetKey, AsyncAssetKeyExt}, asset_url::{AssetType, GetAssetType}, download_asset::{BytesFromUrl, ContentUrl}
};

use crate::{
    track::{AudioFormat, Track}, vorbis::VorbisTrack, Error
};
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AudioFromUrl {
    pub url: ContentUrl,
}

#[async_trait]
impl AsyncAssetKey<Result<Arc<Track>, Arc<Error>>> for AudioFromUrl {
    async fn load(
        self,
        assets: elements_std::asset_cache::AssetCache,
    ) -> Result<Arc<Track>, Arc<Error>>
    where
        Self: 'async_trait,
    {
        let format = match self.url.extension().as_ref().map(|x| x as &str) {
            Some("wav") => AudioFormat::Wav,
            Some("ogg") => AudioFormat::Vorbis,
            v => {
                return Err(Arc::new(Error::UnsupportedFormat(
                    v.unwrap_or_default().to_string(),
                )))
            }
        };
        let bytes: Arc<[u8]> = BytesFromUrl::new(self.url.clone(), true)
            .get(&assets)
            .await
            .map(|v| Arc::from(&v[..]))
            .map_err(|e| Arc::new(e.into()))?;

        Ok(Arc::new(Track::from_format(bytes, format)?))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VorbisFromUrl {
    pub url: ContentUrl,
}

#[async_trait]
impl AsyncAssetKey<Result<Arc<VorbisTrack>, Arc<Error>>> for VorbisFromUrl {
    async fn load(
        self,
        assets: elements_std::asset_cache::AssetCache,
    ) -> Result<Arc<VorbisTrack>, Arc<Error>>
    where
        Self: 'async_trait,
    {
        let bytes: Arc<[u8]> = BytesFromUrl::new(self.url.clone(), true)
            .get(&assets)
            .await
            .map(|v| Arc::from(&v[..]))
            .map_err(|e| Arc::new(e.into()))?;

        Ok(Arc::new(VorbisTrack::new(bytes)?))
    }
}

impl GetAssetType for VorbisTrack {
    fn asset_type() -> AssetType {
        AssetType::VorbisTrack
    }
}
