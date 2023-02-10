use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use kiwi_app::AppBuilder;
use kiwi_asset_timeline::LocalAssetTimelineVisualizer;
use kiwi_cameras::UICamera;
use kiwi_core::{asset_cache, camera::active_camera, runtime};
use kiwi_ecs::World;
use kiwi_element::{ElementComponentExt, Group};
use kiwi_std::asset_cache::{AssetCache, AssetKeepalive, AsyncAssetKey, AsyncAssetKeyExt};
use kiwi_ui::{Button, FocusRoot, WindowSized};
use tokio::task::JoinHandle;

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

fn init(world: &mut World) {
    // load_model(world);
    Group(vec![
        UICamera.el().set(active_camera(), 0.),
        FocusRoot(vec![WindowSized(vec![
            Button::new("Load asset", |world| {
                load_asset(world);
            })
            .el(),
            Button::new("Load and abort asset", |world| load_and_abort_asset(world)).el(),
            Button::new("Load asset (no keepalive)", |world| {
                load_asset_no_keepalive(world);
            })
            .el(),
            Button::new("Load and abort asset (no keepalive)", |world| load_and_abort_asset_no_keepalive(world)).el(),
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
    AppBuilder::simple_ui().run_world(init);
}
