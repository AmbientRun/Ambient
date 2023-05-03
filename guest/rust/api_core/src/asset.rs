use crate::internal::wit;

pub use wit::asset::{AssetCacheStatus, AnimationAssetMetadata};

/// Resolves a asset path for an Ambient asset in this project to an absolute URL.
pub fn url(path: impl AsRef<str>) -> Option<String> {
    wit::asset::url(path.as_ref())
}


/// Peeks the asset cache to prefetch the animation and retrieve its status
pub fn get_animation_asset_status(clip_url: &str) -> AssetCacheStatus {
    wit::asset::get_animation_asset_status(clip_url)
}

/// Peeks the asset cache to retrieve animation metadata if available, such as duration.
pub fn get_animation_asset_metadata(clip_urls: &[&str]) -> Vec<AnimationAssetMetadata> {
    wit::asset::get_animation_asset_metadata(clip_urls)
}


/// Prefetches all animations into the asset cache
pub async fn block_until_animations_are_loaded(clip_urls: &[&str]) {
    crate::prelude::block_until(move || {
        let mut result = true;
        for url in clip_urls {
            if matches!(get_animation_asset_status(url), AssetCacheStatus::NotLoaded) {
                result = false;
            }
        }
        result
    }).await;
}