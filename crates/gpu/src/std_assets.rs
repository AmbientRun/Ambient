use std::sync::Arc;

use ambient_native_std::asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt};
use glam::{uvec4, UVec4};

use crate::{
    gpu::GpuKey,
    texture::{Texture, TextureView},
};

#[derive(Debug, Clone)]
pub struct PixelTextureKey {
    pub colors: Vec<UVec4>,
}
impl PixelTextureKey {
    pub fn white() -> Self {
        Self {
            colors: vec![uvec4(255, 255, 255, 255)],
        }
    }
}
impl SyncAssetKey<Arc<Texture>> for PixelTextureKey {
    fn load(&self, assets: AssetCache) -> Arc<Texture> {
        let gpu = GpuKey.get(&assets);
        Arc::new(Texture::new_single_color_texture_array(
            &gpu,
            self.colors.clone(),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct PixelTextureViewKey {
    pub color: UVec4,
}
impl PixelTextureViewKey {
    pub fn white() -> Self {
        Self {
            color: uvec4(255, 255, 255, 255),
        }
    }
    pub fn black() -> Self {
        Self {
            color: uvec4(0, 0, 0, 255),
        }
    }
}
impl SyncAssetKey<Arc<TextureView>> for PixelTextureViewKey {
    fn load(&self, assets: AssetCache) -> Arc<TextureView> {
        let tex: Arc<Texture> = PixelTextureKey {
            colors: vec![self.color],
        }
        .get(&assets);
        Arc::new(tex.create_view(&Default::default()))
    }
}

#[derive(Debug, Clone)]
pub struct DefaultNormalMapKey;
impl SyncAssetKey<Arc<Texture>> for DefaultNormalMapKey {
    fn load(&self, assets: AssetCache) -> Arc<Texture> {
        let gpu = GpuKey.get(&assets);
        Arc::new(Texture::new_single_color_texture_array(
            &gpu,
            vec![uvec4(128, 128, 255, 0)],
        ))
    }
}

#[derive(Debug, Clone)]
pub struct DefaultNormalMapViewKey;
impl SyncAssetKey<Arc<TextureView>> for DefaultNormalMapViewKey {
    fn load(&self, assets: AssetCache) -> Arc<TextureView> {
        let tex: Arc<Texture> = DefaultNormalMapKey.get(&assets);
        Arc::new(tex.create_view(&Default::default()))
    }
}

#[derive(Debug, Clone)]
pub struct PixelTextureArrayViewKey {
    pub colors: Vec<UVec4>,
}
impl SyncAssetKey<Arc<TextureView>> for PixelTextureArrayViewKey {
    fn load(&self, assets: AssetCache) -> Arc<TextureView> {
        let tex: Arc<Texture> = PixelTextureKey {
            colors: self.colors.clone(),
        }
        .get(&assets);
        Arc::new(tex.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        }))
    }
}

#[deprecated(note = "Use PixelTextureKey")]
pub fn get_pixel_texture(assets: AssetCache, color: UVec4) -> Arc<Texture> {
    PixelTextureKey {
        colors: vec![color],
    }
    .get(&assets)
}
#[deprecated(note = "Use PixelTextureKey")]
pub fn get_pixel_texture_array(assets: AssetCache, colors: Vec<UVec4>) -> Arc<Texture> {
    PixelTextureKey { colors }.get(&assets)
}

#[deprecated(note = "Use PixelTextureViewKey")]
pub fn get_pixel_texture_view(assets: AssetCache, color: UVec4) -> Arc<TextureView> {
    PixelTextureViewKey { color }.get(&assets)
}
#[deprecated(note = "Use PixelTextureArrayViewKey")]
pub fn get_pixel_texture_array_view(assets: AssetCache, colors: Vec<UVec4>) -> Arc<TextureView> {
    PixelTextureArrayViewKey { colors }.get(&assets)
}
