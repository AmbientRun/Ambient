use anyhow::Context;
use elements_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl, download_asset::download};
use image::ImageFormat;
use serde::de::DeserializeOwned;

pub async fn download_bytes(assets: &AssetCache, url: &AbsAssetUrl) -> anyhow::Result<Vec<u8>> {
    Ok(download(assets, url.0.clone(), |resp| async move { Ok(resp.bytes().await?) }).await?.into())
}

pub async fn download_text(assets: &AssetCache, url: &AbsAssetUrl) -> anyhow::Result<String> {
    download(assets, url.0.clone(), |resp| async move { Ok(resp.text().await?) }).await
}

pub async fn download_json<T: DeserializeOwned>(assets: &AssetCache, url: &AbsAssetUrl) -> anyhow::Result<T> {
    download(assets, url.0.clone(), |resp| async move { Ok(resp.json::<T>().await?) }).await
}

pub async fn download_image(assets: &AssetCache, url: &AbsAssetUrl, extension: &Option<String>) -> anyhow::Result<image::DynamicImage> {
    let data = download_bytes(assets, url).await?;
    if let Some(format) = extension.as_ref().and_then(|ext| ImageFormat::from_extension(ext)) {
        Ok(image::load_from_memory_with_format(&data, format).with_context(|| format!("Failed to load image {url}"))?)
    } else {
        Ok(image::load_from_memory(&data).with_context(|| format!("Failed to load image {url}"))?)
    }
}
