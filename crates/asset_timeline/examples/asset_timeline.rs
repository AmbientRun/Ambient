use std::{sync::Arc, time::Duration};

use ambient_app::{App, AppBuilder};
use ambient_asset_timeline::LocalAssetTimelineVisualizer;
use ambient_cameras::UICamera;
use ambient_core::{asset_cache, runtime};
use ambient_ecs::World;
use ambient_element::{ElementComponentExt, Group};
use ambient_std::asset_cache::{AssetCache, AssetKeepalive, AsyncAssetKey, AsyncAssetKeyExt};
use ambient_sys::task::JoinHandle;
use ambient_ui_native::{Button, FocusRoot, WindowSized};
use async_trait::async_trait;

#[derive(PartialEq, Eq, Debug)]
struct NoKeepalive;

#[derive(Debug, Clone)]
struct NoKeepaliveKey;

#[async_trait]
impl AsyncAssetKey<Arc<NoKeepalive>> for NoKeepaliveKey {
    async fn load(self, _: AssetCache) -> Arc<NoKeepalive> {
        tokio::time::sleep(Duration::from_secs(5)).await;

        Arc::new(NoKeepalive)
    }

    fn keepalive(&self) -> AssetKeepalive {
        AssetKeepalive::None
    }
}

#[derive(PartialEq, Eq, Debug)]
struct TestAsset;

#[derive(Debug, Clone)]
struct TestAssetKey;

#[async_trait]
impl AsyncAssetKey<Arc<TestAsset>> for TestAssetKey {
    async fn load(self, _: AssetCache) -> Arc<TestAsset> {
        tokio::time::sleep(Duration::from_secs(5)).await;

        Arc::new(TestAsset)
    }

    fn keepalive(&self) -> AssetKeepalive {
        AssetKeepalive::Timeout(Duration::from_secs(30))
    }
}

fn load_asset(world: &mut World) -> JoinHandle<()> {
    let assets = world.resource(asset_cache()).clone();
    world.resource(runtime()).spawn(async move {
        TestAssetKey.get(&assets).await;
        tracing::info!("Finished loading asset");
    })
}

fn load_and_abort_asset(world: &mut World) {
    let task = load_asset(world);
    world.resource(runtime()).spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        task.abort();
    });
}

fn load_asset_no_keepalive(world: &mut World) -> JoinHandle<()> {
    let assets = world.resource(asset_cache()).clone();
    world.resource(runtime()).spawn(async move {
        let asset = NoKeepaliveKey.get(&assets).await;
        tracing::info!("Finished loading asset");
        tokio::time::sleep(Duration::from_secs(5)).await;
        drop(asset);
    })
}

fn load_and_abort_asset_no_keepalive(world: &mut World) {
    let task = load_asset_no_keepalive(world);
    world.resource(runtime()).spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        task.abort();
    });
}

async fn init(app: &mut App) {
    let world = &mut app.world;
    // load_model(world);
    Group(vec![
        UICamera.el(),
        FocusRoot(vec![WindowSized(vec![
            Button::new("Load asset", |world| {
                load_asset(world);
            })
            .el(),
            Button::new("Load and abort asset", load_and_abort_asset).el(),
            Button::new("Load asset (no keepalive)", |world| {
                load_asset_no_keepalive(world);
            })
            .el(),
            Button::new("Load and abort asset (no keepalive)", load_and_abort_asset_no_keepalive).el(),
            LocalAssetTimelineVisualizer.el(),
        ])
        .el()])
        .el(),
    ])
    .el()
    .spawn_interactive(world);
}

fn main() {
    tracing_subscriber::fmt().init();
    AppBuilder::simple_ui().block_on(init)
}
