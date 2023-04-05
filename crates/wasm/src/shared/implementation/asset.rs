use ambient_std::{
    asset_cache::SyncAssetKeyExt,
    asset_url::{AssetUrl, ServerBaseUrlKey, ProxyBaseUrlKey},
};

use ambient_core::{
    asset_cache
};

use ambient_ecs::World;

pub(crate) fn url(
    world: &mut World,
    path: String,
) -> anyhow::Result<Option<String>> {
    let assets = world.resource(asset_cache());
    let base_url = ProxyBaseUrlKey.try_get(assets).unwrap_or_else(|| ServerBaseUrlKey.get(assets));
    Ok(Some(AssetUrl::parse(path)?.resolve(&base_url)?.to_string()))
    // Ok(Some(String::from("42")))
}