use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use elements_asset_cache::{AssetCache, AssetKeepalive, AsyncAssetKey, AsyncAssetKeyExt};
use futures::FutureExt;
use pretty_assertions::assert_eq;
use tokio::{runtime, time::sleep};

#[derive(PartialEq, Eq, Debug)]
struct TestAsset;

#[derive(Debug, Clone)]
struct TestAssetKey;

#[async_trait]
impl AsyncAssetKey<Arc<TestAsset>> for TestAssetKey {
    async fn load(self, _: AssetCache) -> Arc<TestAsset> {
        tokio::time::sleep(Duration::from_secs(1)).await;

        Arc::new(TestAsset)
    }

    fn keepalive(&self) -> AssetKeepalive {
        AssetKeepalive::Timeout(Duration::from_secs(5))
    }
}

#[tokio::test]
async fn keepalive() {
    let assets = AssetCache::new(runtime::Handle::current());

    {
        let asset = TestAssetKey.get(&assets).await;
        assert_eq!(&*asset, &TestAsset);
    }

    // Keepalive
    sleep(Duration::from_secs(2)).await;
    assert_eq!(TestAssetKey.is_loaded(&assets), Some(Arc::new(TestAsset)));

    // Keepalive ended
    sleep(Duration::from_secs(4)).await;
    assert_eq!(TestAssetKey.is_loaded(&assets), None);
}

#[tokio::test]
async fn keepalive_again() {
    let assets = AssetCache::new(runtime::Handle::current());

    {
        let asset = TestAssetKey.get(&assets).await;
        assert_eq!(&*asset, &TestAsset);
    }

    // Keepalive
    sleep(Duration::from_secs(2)).await;
    assert_eq!(TestAssetKey.is_loaded(&assets), Some(Arc::new(TestAsset)));
    let asset = {
        let asset = TestAssetKey.get(&assets).now_or_never();
        assert_eq!(asset.as_deref(), Some(&TestAsset));
        Arc::downgrade(&asset.unwrap())
    };

    // Keepalive is still active
    sleep(Duration::from_secs(4)).await;
    assert_eq!(TestAssetKey.is_loaded(&assets), Some(asset.upgrade().unwrap()));
}

#[derive(PartialEq, Eq, Debug)]
struct NK;

#[derive(Debug, Clone)]
struct NKKey;

#[async_trait]
impl AsyncAssetKey<Arc<NK>> for NKKey {
    async fn load(self, _: AssetCache) -> Arc<NK> {
        tokio::time::sleep(Duration::from_secs(1)).await;

        Arc::new(NK)
    }

    fn keepalive(&self) -> AssetKeepalive {
        AssetKeepalive::None
    }
}

#[tokio::test]
async fn no_keepalive() {
    let assets = AssetCache::new(runtime::Handle::current());

    {
        let asset = NKKey.get(&assets).await;
        assert_eq!(&*asset, &NK);
    }

    // Keepalive
    sleep(Duration::from_secs(2)).await;
    assert_eq!(NKKey.is_loaded(&assets), None);

    sleep(Duration::from_secs(4)).await;
    assert_eq!(NKKey.is_loaded(&assets), None);
}

#[tokio::test]
async fn no_keepalive_again() {
    let assets = AssetCache::new(runtime::Handle::current());

    {
        let asset = NKKey.get(&assets).await;
        assert_eq!(&*asset, &NK);
    }

    // Keepalive
    sleep(Duration::from_secs(2)).await;
    assert_eq!(NKKey.is_loaded(&assets), None);
    {
        let asset = NKKey.get(&assets).now_or_never();
        assert_eq!(asset.as_deref(), None);
    };

    sleep(Duration::from_secs(4)).await;
    assert_eq!(NKKey.is_loaded(&assets), None);
}
