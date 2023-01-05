use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use elements_ecs::{query_mut, DeserWorldWithWarnings, World};
use elements_model::{model_def, ModelDef};
use elements_physics::collider::collider;
use elements_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt}, asset_url::AbsAssetUrl, download_asset::{AssetError, BytesFromUrl}
};

#[derive(Debug, Clone)]
pub struct ObjectFromUrl(pub AbsAssetUrl);
#[async_trait]
impl AsyncAssetKey<Result<Arc<World>, AssetError>> for ObjectFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<World>, AssetError> {
        let data = BytesFromUrl::new(self.0.clone(), true).get(&assets).await?;
        let DeserWorldWithWarnings { mut world, warnings } = tokio::task::block_in_place(|| serde_json::from_slice(&data))
            .with_context(|| format!("Failed to deserialize object2 from url {}", self.0))?;
        warnings.log_warnings();
        for (id, (url,), _) in query_mut((model_def(),), ()).iter(&mut world, None) {
            *url = ModelDef(url.0.resolve(&self.0).context("Failed to resolve model url")?.into());
        }
        for (id, (def,), _) in query_mut((collider(),), ()).iter(&mut world, None) {
            def.resolve(&self.0).context("Failed to resolve collider")?;
        }
        Ok(Arc::new(world))
    }
}
