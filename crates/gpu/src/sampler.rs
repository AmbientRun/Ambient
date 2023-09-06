use std::sync::Arc;

use ambient_native_std::asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt};
use serde::{Deserialize, Serialize};

use crate::gpu::GpuKey;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplerKey {
    pub address_mode_u: wgpu::AddressMode,
    pub address_mode_v: wgpu::AddressMode,
    pub address_mode_w: wgpu::AddressMode,
    pub mag_filter: wgpu::FilterMode,
    pub min_filter: wgpu::FilterMode,
    pub mipmap_filter: wgpu::FilterMode,
}

impl SamplerKey {
    pub const LINEAR_CLAMP_TO_EDGE: SamplerKey = SamplerKey {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
    };
    pub const LINEAR_REPEAT: SamplerKey = SamplerKey {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
    };
}

impl SyncAssetKey<Arc<wgpu::Sampler>> for SamplerKey {
    fn load(&self, assets: AssetCache) -> Arc<wgpu::Sampler> {
        let gpu = GpuKey.get(&assets);
        Arc::new(gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: self.address_mode_u,
            address_mode_v: self.address_mode_v,
            address_mode_w: self.address_mode_w,
            mag_filter: self.mag_filter,
            min_filter: self.min_filter,
            mipmap_filter: self.mipmap_filter,
            ..wgpu::SamplerDescriptor::default()
        }))
    }
}
