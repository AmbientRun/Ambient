use async_trait::async_trait;

use crate::{Asset, AssetCache, AssetLoadDropPolicy, AsyncAssetKey, AsyncAssetKeyExt};

/// A key wrapper which will force the loading to happen in a non interruptable task.
///
/// This is useful to prevent the loading from being aborted and having to restart for E.g;
/// short lived UI components.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct BackgroundKey<K>(pub(crate) K);

// /// Make sure the background key maps to the same K
// impl<K> std::fmt::Debug for BackgroundKey<K>
// where
//     K: std::fmt::Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.0.fmt(f)
//     }
// }

#[async_trait]
impl<K, T> AsyncAssetKey<T> for BackgroundKey<K>
where
    K: 'static + Clone + AsyncAssetKey<T>,
    T: 'static + Clone + Asset + Sync + Send,
{
    fn drop_policy(&self) -> AssetLoadDropPolicy {
        AssetLoadDropPolicy::KeepLoading
    }

    async fn load(self, assets: AssetCache) -> T {
        // Short happy path
        // This is needed as JoinHandle does not complete immediately, even if the spawned future
        // is ready
        if let Some(content) = assets.content_state(&self.0) {
            if let Some(value) = content.get_loaded_value::<T>() {
                return value;
            }
        }

        let key = self.0.clone();
        let runtime = assets.runtime().clone();
        let task = runtime.spawn(async move {
            let v = key.get(&assets).await;
            v
        });

        task.await.expect("Failed to wait for load task")
    }
}
