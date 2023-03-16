use ambient_core::{asset_cache, runtime};
use ambient_element::Hooks;
use ambient_std::asset_cache::{Asset, AsyncAssetKeyExt};

pub fn use_async_asset<T: Asset + Clone + Sync + Send + std::fmt::Debug + 'static>(
    hooks: &mut Hooks,
    asset_key: impl AsyncAssetKeyExt<T> + 'static,
) -> Option<T> {
    let (value, set_value) = hooks.use_state(None);
    hooks.use_effect(asset_key.key(), |world, _| {
        let assets = world.resource(asset_cache()).clone();
        world.resource(runtime()).spawn(async move {
            set_value(Some(asset_key.get(&assets).await));
        });
        Box::new(|_| {})
    });
    value
}
