use std::time::Duration;

use elements_core::{asset_cache, runtime};
use elements_ecs::World;
use elements_element::Hooks;
use elements_std::asset_cache::{Asset, AsyncAssetKeyExt};

pub fn use_interval<F: Fn() + Sync + Send + 'static>(hooks: &mut Hooks, seconds: f32, cb: F) {
    hooks.use_spawn(move |world| {
        let thread = world.resource(runtime()).spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs_f32(seconds));
            interval.tick().await;
            loop {
                interval.tick().await;
                cb();
            }
        });
        Box::new(move |_| {
            thread.abort();
        })
    });
}

pub fn use_interval_deps<F: Fn() + Sync + Send + 'static, D: PartialEq + Clone + Sync + Send + std::fmt::Debug + 'static>(
    world: &mut World,
    hooks: &mut Hooks,
    dependencies: D,
    seconds: f32,
    cb: F,
) {
    hooks.use_effect(world, dependencies, move |world| {
        let thread = world.resource(runtime()).spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs_f32(seconds));
            interval.tick().await;
            loop {
                interval.tick().await;
                cb();
            }
        });
        Box::new(move |_| {
            thread.abort();
        })
    });
}

pub fn use_async_asset<T: Asset + Clone + Sync + Send + std::fmt::Debug + 'static>(
    hooks: &mut Hooks,
    world: &mut World,
    asset_key: impl AsyncAssetKeyExt<T> + 'static,
) -> Option<T> {
    let (value, set_value) = hooks.use_state(None);
    hooks.use_effect(world, asset_key.key(), |world| {
        let assets = world.resource(asset_cache()).clone();
        world.resource(runtime()).spawn(async move {
            set_value(Some(asset_key.get(&assets).await));
        });
        Box::new(|_| {})
    });
    value
}
