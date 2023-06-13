use std::{sync::Arc, time::Duration};

use ambient_asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt};
use async_trait::async_trait;
use futures::FutureExt;
use pretty_assertions::assert_eq;
use tokio::{
    runtime,
    time::{sleep, timeout},
};

#[derive(PartialEq, Eq, Debug)]
struct TestAsset {
    name: String,
}

#[derive(Debug, Clone)]
struct TestAssetKey {
    name: String,
}

#[async_trait]
impl AsyncAssetKey<Arc<TestAsset>> for TestAssetKey {
    async fn load(self, _: AssetCache) -> Arc<TestAsset> {
        tokio::time::sleep(Duration::from_secs(1)).await;

        Arc::new(TestAsset { name: self.name })
    }
}

#[tokio::test]
async fn load_asset_async() {
    let assets = AssetCache::new(runtime::Handle::current());

    let asset = TestAssetKey { name: "foo".into() }.get(&assets).await;

    assert_eq!(&*asset, &TestAsset { name: "foo".into() })
}

#[tokio::test]
async fn load_aborted() {
    let assets = AssetCache::new(runtime::Handle::current());

    let asset = timeout(
        Duration::from_millis(200),
        TestAssetKey { name: "foo".into() }.get(&assets),
    )
    .await;

    assert!(asset.is_err());

    let a = TestAssetKey { name: "foo".into() }.get(&assets).await;

    let b = TestAssetKey { name: "foo".into() }
        .get(&assets)
        .now_or_never()
        .unwrap();
    assert_eq!(&*b, &TestAsset { name: "foo".into() });
    assert!(Arc::ptr_eq(&a, &b));
}

#[tokio::test]
async fn load_shared() {
    let assets = AssetCache::new(runtime::Handle::current());

    let a = TestAssetKey { name: "foo".into() };
    let a = a.get(&assets);

    let b = TestAssetKey { name: "foo".into() };
    let mut b = b.get(&assets);

    assert_eq!((&mut b).now_or_never(), None);

    assert_eq!(a.now_or_never(), None);

    let asset = b.await;
    assert_eq!(&*asset, &TestAsset { name: "foo".into() });
}

#[tokio::test]
async fn peek() {
    let assets = AssetCache::new(runtime::Handle::current());

    let key = TestAssetKey { name: "foo".into() };

    assert_eq!(key.peek(&assets), None);

    sleep(Duration::from_secs(2)).await;

    assert_eq!(
        key.peek(&assets).as_deref(),
        Some(&TestAsset { name: "foo".into() })
    );
}

#[tokio::test]
async fn peek_and_get() {
    let assets = AssetCache::new(runtime::Handle::current());

    let key = TestAssetKey { name: "foo".into() };
    // This starts a future for loading
    let a = key.get(&assets);

    // Peeking spawn a task which will advance *the same* future as `a`
    assert_eq!(key.peek(&assets), None);

    sleep(Duration::from_secs(2)).await;

    let a = a.now_or_never().unwrap();

    let b = key.peek(&assets).unwrap();

    // Only one instance should be loaded
    assert!(Arc::ptr_eq(&a, &b))
}

#[tokio::test]
async fn peek_and_drop() {
    let assets = AssetCache::new(runtime::Handle::current());

    let key = TestAssetKey { name: "foo".into() };
    // This starts a future for loading
    let mut a = key.get(&assets);

    // poll once
    assert_eq!((&mut a).now_or_never(), None);

    // Peeking spawn a task which will advance *the same* future as `a`
    assert_eq!(key.peek(&assets), None);

    // Even if a is dropped, the same future is still polled by the task spawned by peek
    drop(a);

    sleep(Duration::from_secs(2)).await;

    let asset = key.peek(&assets).unwrap();
    assert_eq!(&*asset, &TestAsset { name: "foo".into() });
}

#[tokio::test]
async fn peek_in_blocking() {
    let assets = AssetCache::new(runtime::Handle::current());

    let asset = tokio::task::spawn_blocking(move || {
        for _ in 0..100 {
            let asset = TestAssetKey { name: "foo".into() }.peek(&assets);
            if let Some(asset) = asset {
                return asset;
            }

            std::thread::sleep(Duration::from_millis(200));
        }

        panic!("Peek did not complete");
    })
    .await
    .unwrap();

    assert_eq!(&*asset, &TestAsset { name: "foo".into() });
}

#[tokio::test]
async fn get_in_blocking() {
    let assets = AssetCache::new(runtime::Handle::current());

    let asset = tokio::task::spawn_blocking(move || {
        for _ in 0..100 {
            let asset = TestAssetKey { name: "foo".into() }
                .in_background()
                .get(&assets)
                .now_or_never();
            if let Some(asset) = asset {
                return asset;
            }

            std::thread::sleep(Duration::from_millis(200));
        }

        panic!("Peek did not complete");
    })
    .await
    .unwrap();

    assert_eq!(&*asset, &TestAsset { name: "foo".into() });
}

#[tokio::test]
async fn load_background() {
    let assets = AssetCache::new(runtime::Handle::current());

    let key = TestAssetKey { name: "foo".into() };

    let a = key.get(&assets).await;

    // Make sure BackgroundKey and Key resolve to the same asset
    let b = key.in_background().get(&assets).now_or_never().unwrap();

    assert_eq!(&*b, &TestAsset { name: "foo".into() });

    assert!(Arc::ptr_eq(&a, &b));
}
