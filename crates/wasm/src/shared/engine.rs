use std::sync::Arc;

use ambient_native_std::asset_cache::SyncAssetKey;
use anyhow::Context;

#[derive(Clone)]
// JIT execution
pub struct Engine {
    engine: wasm_bridge::Engine,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine").finish()
    }
}

#[derive(Debug, Clone)]
pub struct EngineKey;

impl SyncAssetKey<Result<Engine, Arc<anyhow::Error>>> for EngineKey {
    fn load(
        &self,
        _assets: ambient_native_std::asset_cache::AssetCache,
    ) -> Result<Engine, Arc<anyhow::Error>> {
        let mut config = wasm_bridge::Config::new();
        config.debug_info(true);

        Ok(Engine {
            engine: wasm_bridge::Engine::new(&config)
                .context("Failed to create wasm execution engine")
                .map_err(Arc::new)?,
        })
    }
}
