use std::sync::Arc;

use ambient_pipeline_types::materials::SamplerDesc;
use ambient_std::asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt};
use serde::{Deserialize, Serialize};

use crate::gpu::GpuKey;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplerKey(pub SamplerDesc);

impl SamplerKey {
    pub const LINEAR_CLAMP_TO_EDGE: Self = Self(SamplerDesc::LINEAR_CLAMP_TO_EDGE);
}

impl SyncAssetKey<Arc<wgpu::Sampler>> for SamplerKey {
    fn load(&self, assets: AssetCache) -> Arc<wgpu::Sampler> {
        let gpu = GpuKey.get(&assets);
        Arc::new(gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: self.0.address_mode_u,
            address_mode_v: self.0.address_mode_v,
            address_mode_w: self.0.address_mode_w,
            mag_filter: self.0.mag_filter,
            min_filter: self.0.min_filter,
            mipmap_filter: self.0.mipmap_filter,
            ..wgpu::SamplerDescriptor::default()
        }))
    }
}
