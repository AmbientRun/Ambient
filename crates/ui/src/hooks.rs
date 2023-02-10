use std::{fmt::Debug, time::Duration};

use kiwi_core::{asset_cache, runtime};
use kiwi_ecs::World;
use kiwi_element::Hooks;
use kiwi_std::asset_cache::{Asset, AsyncAssetKeyExt};

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

pub fn use_interval_deps<D>(
    world: &mut World,
    hooks: &mut Hooks,
    duration: Duration,
    run_immediately: bool,
    dependencies: D,
    mut func: impl 'static + Send + Sync + FnMut(&D),
) where
    D: 'static + Send + Sync + Clone + Debug + PartialEq,
{
    hooks.use_effect(world, dependencies.clone(), move |world, _| {
        if run_immediately {
            func(&dependencies);
        }

        let task = world.resource(runtime()).spawn(async move {
            let mut interval = tokio::time::interval(duration);
            interval.tick().await;
            loop {
                interval.tick().await;
                func(&dependencies);
            }
        });

        Box::new(move |_| {
            task.abort();
        })
    });
}

pub fn use_async_asset<T: Asset + Clone + Sync + Send + std::fmt::Debug + 'static>(
    hooks: &mut Hooks,
    world: &mut World,
    asset_key: impl AsyncAssetKeyExt<T> + 'static,
) -> Option<T> {
    let (value, set_value) = hooks.use_state(None);
    hooks.use_effect(world, asset_key.key(), |world, _| {
        let assets = world.resource(asset_cache()).clone();
        world.resource(runtime()).spawn(async move {
            set_value(Some(asset_key.get(&assets).await));
        });
        Box::new(|_| {})
    });
    value
}
