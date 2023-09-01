use std::{borrow::Cow, fmt, io::Cursor, sync::Arc};

use ambient_native_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    download_asset::{AssetError, AssetResult, BytesFromUrl},
    CowStr,
};
use ambient_sys::task;
use async_trait::async_trait;
use futures::future::join_all;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};

use crate::{gpu::GpuKey, texture::Texture};

#[derive(Debug, Clone)]
pub struct ImageFromUrl {
    pub url: AbsAssetUrl,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<DynamicImage>, AssetError>> for ImageFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<DynamicImage>, AssetError> {
        image_from_url(assets, self.url).await.map(Arc::new)
    }
}

#[derive(Debug, Clone)]
pub struct Rgba8ImageFromUrl {
    pub url: AbsAssetUrl,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<image::RgbaImage>, AssetError>> for Rgba8ImageFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<image::RgbaImage>, AssetError> {
        image_from_url(assets, self.url)
            .await
            .map(|x| Arc::new(x.into_rgba8()))
    }
}

async fn image_from_url(assets: AssetCache, url: AbsAssetUrl) -> Result<DynamicImage, AssetError> {
    let data = BytesFromUrl::new(url.clone(), true).get(&assets).await?;

    Ok(task::block_in_place({
            let url = url.clone();
            move || -> anyhow::Result<DynamicImage> {
                if let Some(format) = url.extension().and_then(ImageFormat::from_extension) {
                    Ok(image::io::Reader::with_format(Cursor::new(&*data), format).decode()?)
                } else {
                    Ok(image::io::Reader::new(Cursor::new(&*data))
                        .with_guessed_format()
                        .with_context(|| format!("Failed to guess format, and couldn't find an extension of image with URL \"{url}\""))?
                        .decode()?)
                }
        }})
        .with_context(|| format!("Failed to load image from \"{url}\""))?)
}

#[derive(Debug, Clone)]
pub struct TextureFromUrl {
    pub url: AbsAssetUrl,
    pub format: wgpu::TextureFormat,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<Texture>, AssetError>> for TextureFromUrl {
    fn gpu_size(&self, asset: &Result<Arc<Texture>, AssetError>) -> Option<u64> {
        asset.as_ref().ok().map(|asset| asset.size_in_bytes)
    }
    #[tracing::instrument(level = "info", name = "texture_from_url")]
    async fn load(self, assets: AssetCache) -> Result<Arc<Texture>, AssetError> {
        let gpu = GpuKey.get(&assets);
        let image = image_from_url(assets.clone(), self.url.clone()).await?;
        task::block_in_place(|| {
            Ok(Arc::new(Texture::from_image_mipmapped(
                &gpu,
                &assets,
                image,
                self.format,
                Some(&self.url.to_string()),
            )))
        })
    }
}

#[derive(Clone, Debug)]
pub struct TextureFromRgba8Image {
    pub image: Arc<dyn AsyncAssetKeyExt<Result<Arc<image::RgbaImage>, AssetError>>>,
    pub format: wgpu::TextureFormat,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<Texture>, AssetError>> for TextureFromRgba8Image {
    fn gpu_size(&self, asset: &Result<Arc<Texture>, AssetError>) -> Option<u64> {
        asset.as_ref().ok().map(|x| x.size_in_bytes)
    }
    async fn load(self, assets: AssetCache) -> Result<Arc<Texture>, AssetError> {
        let gpu = GpuKey.get(&assets);
        let img = self.image.get(&assets).await?;
        task::block_in_place(|| {
            Ok(Arc::new(Texture::from_rgba8_image_mipmapped(
                &gpu,
                &assets,
                &img,
                self.format,
                Some(&format!("{:?}", self.image)),
            )))
        })
    }
}

#[derive(Debug, Clone)]
pub struct TextureFromBytes {
    bytes: Cow<'static, [u8]>,
    label: Option<CowStr>,
}

impl TextureFromBytes {
    pub fn new(bytes: impl Into<Cow<'static, [u8]>>, label: Option<impl Into<CowStr>>) -> Self {
        Self {
            bytes: bytes.into(),
            label: label.map(Into::into),
        }
    }
}

use anyhow::Context;

#[async_trait]
impl AsyncAssetKey<Result<Arc<Texture>, AssetError>> for TextureFromBytes {
    fn gpu_size(&self, asset: &Result<Arc<Texture>, AssetError>) -> Option<u64> {
        asset.as_ref().ok().map(|asset| asset.size_in_bytes)
    }
    async fn load(self, assets: AssetCache) -> Result<Arc<Texture>, AssetError> {
        let gpu = GpuKey.get(&assets);
        let texture = task::spawn_blocking(move || -> anyhow::Result<Arc<Texture>> {
            let image = image::load_from_memory(&self.bytes[..])
                .context("Failed to load image from bytes")?;
            Ok(Arc::new(Texture::from_image_mipmapped(
                &gpu,
                &assets,
                image,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                self.label.as_deref(),
            )))
        })
        .await
        .context("Failed to join")??;

        Ok(texture)
    }
}

#[derive(Clone)]
pub struct Rgba8ImageInMemory {
    pub image_uid: String,
    pub image: Arc<image::RgbaImage>,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<image::RgbaImage>, AssetError>> for Rgba8ImageInMemory {
    async fn load(self, _assets: AssetCache) -> Result<Arc<image::RgbaImage>, AssetError> {
        Ok(self.image)
    }
}
impl std::fmt::Debug for Rgba8ImageInMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rgba8ImageInMemory")
            .field("image_uid", &self.image_uid)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct SplitImageFromUrl {
    pub color: AbsAssetUrl,
    pub alpha: AbsAssetUrl,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<image::RgbaImage>, AssetError>> for SplitImageFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<image::RgbaImage>, AssetError> {
        let color = image_from_url(assets.clone(), self.color.clone()).await?;
        let alpha = image_from_url(assets.clone(), self.alpha.clone()).await?;
        task::block_in_place(|| {
            let mut color = color.into_rgba8();
            let alpha = alpha.into_luma8();
            for (color, alpha) in color.pixels_mut().zip(alpha.pixels()) {
                color.0[3] = alpha.0[0];
            }
            Ok(Arc::new(color))
        })
    }
}

#[derive(Debug, Clone)]
pub struct SplitTextureFromUrl {
    pub color: AbsAssetUrl,
    pub alpha: AbsAssetUrl,
    pub format: wgpu::TextureFormat,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<Texture>, AssetError>> for SplitTextureFromUrl {
    fn gpu_size(&self, asset: &Result<Arc<Texture>, AssetError>) -> Option<u64> {
        asset.as_ref().ok().map(|asset| asset.size_in_bytes)
    }
    async fn load(self, assets: AssetCache) -> Result<Arc<Texture>, AssetError> {
        let gpu = GpuKey.get(&assets);
        let color = image_from_url(assets.clone(), self.color.clone()).await?;
        let alpha = image_from_url(assets.clone(), self.alpha.clone()).await?;
        task::block_in_place(|| {
            let mut color = color.into_rgba8();
            let alpha = alpha.into_luma8();
            for (color, alpha) in color.pixels_mut().zip(alpha.pixels()) {
                color.0[3] = alpha.0[0];
            }
            let label = format!("color={} alpha={}", self.color, self.alpha);
            Ok(Arc::new(Texture::from_image_mipmapped(
                &gpu,
                &assets,
                DynamicImage::ImageRgba8(color),
                self.format,
                Some(&label),
            )))
        })
    }
}

#[derive(Debug, Clone)]
pub struct Rgba8ImageCappedSize {
    pub image: Arc<dyn AsyncAssetKeyExt<Result<Arc<image::RgbaImage>, AssetError>>>,
    pub max_size: u32,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<image::RgbaImage>, AssetError>> for Rgba8ImageCappedSize {
    async fn load(self, assets: AssetCache) -> Result<Arc<image::RgbaImage>, AssetError> {
        let image = self.image.get(&assets).await?;
        if image.width() > self.max_size || image.height() > self.max_size {
            let (width, height) = if image.width() >= image.height() {
                (
                    self.max_size,
                    (self.max_size as f32 * image.height() as f32 / image.width() as f32) as u32,
                )
            } else {
                (
                    (self.max_size as f32 * image.width() as f32 / image.height() as f32) as u32,
                    self.max_size,
                )
            };
            Ok(Arc::new(image::imageops::resize(
                &*image as &image::RgbaImage,
                width,
                height,
                image::imageops::FilterType::CatmullRom,
            )))
        } else {
            Ok(image)
        }
    }
}

// #[deprecated(
//     note = "Warning; we haven't verified that doing Debug on a func actually yields unique results for different functions, so this may not work."
// )]
#[derive(Clone)]
pub struct MappedTexture<F> {
    inner: TextureFromUrl,
    func: F,
}

impl<F> fmt::Debug for MappedTexture<F>
where
    F: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MappedTexture")
            .field("inner", &format!("{:?}", std::any::TypeId::of::<F>()))
            .finish()
    }
}

impl<F> MappedTexture<F>
where
    F: Fn(Rgba<u8>) -> Rgba<u8> + Send + Sync,
{
    pub fn new(inner: TextureFromUrl, func: F) -> Self {
        Self { inner, func }
    }
}

#[async_trait]
impl<F> AsyncAssetKey<Result<Arc<Texture>, AssetError>> for MappedTexture<F>
where
    F: Fn(Rgba<u8>) -> Rgba<u8> + Send + Sync + 'static,
{
    fn gpu_size(&self, asset: &Result<Arc<Texture>, AssetError>) -> Option<u64> {
        asset.as_ref().ok().map(|asset| asset.size_in_bytes)
    }
    async fn load(self, assets: AssetCache) -> Result<Arc<Texture>, AssetError> {
        let mut image = image_from_url(assets.clone(), self.inner.url.clone())
            .await?
            .into_rgba8();
        image.pixels_mut().for_each(|v| (*v = (self.func)(*v)));

        let gpu = GpuKey.get(&assets);
        task::block_in_place(|| {
            Ok(Arc::new(Texture::from_image_mipmapped(
                &gpu,
                &assets,
                DynamicImage::ImageRgba8(image),
                self.inner.format,
                Some(&self.inner.url.to_string()),
            )))
        })
    }
}

#[derive(Debug, Clone)]
pub struct TextureArrayFromUrls {
    pub urls: Vec<AbsAssetUrl>,
    pub format: wgpu::TextureFormat,
    pub label: Option<String>,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<Texture>, AssetError>> for TextureArrayFromUrls {
    fn gpu_size(&self, asset: &Result<Arc<Texture>, AssetError>) -> Option<u64> {
        asset.as_ref().ok().map(|asset| asset.size_in_bytes)
    }
    async fn load(self, assets: AssetCache) -> Result<Arc<Texture>, AssetError> {
        let texs = join_all(
            self.urls
                .into_iter()
                .map(|url| {
                    let assets = assets.clone();
                    async move {
                        let data = BytesFromUrl::new(url.clone(), true).get(&assets).await?;
                        task::block_in_place(|| -> anyhow::Result<RgbaImage> {
                            Ok(image::io::Reader::new(Cursor::new(&*data))
                                .with_guessed_format()
                                .with_context(|| format!("Failed to guess format of {url}"))?
                                .decode()
                                .with_context(|| format!("Failed to decode image from {url}"))?
                                .into_rgba8())
                        })
                    }
                })
                .collect::<Vec<_>>(),
        )
        .await;
        let gpu = GpuKey.get(&assets);
        task::block_in_place(|| -> AssetResult<Arc<Texture>> {
            let imgs = texs
                .into_iter()
                .collect::<anyhow::Result<Vec<RgbaImage>>>()?;
            Ok(Arc::new(Texture::array_rgba8_mipmapped(
                &gpu,
                &assets,
                self.label.as_ref().map(|x| x as &str),
                imgs,
                self.format,
            )))
        })
    }
}
