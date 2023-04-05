use ambient_std::{
    asset_url::{AbsAssetUrl, AssetUrl}
};

use ambient_network::abs_asset_url;
use ambient_ecs::World;

pub(crate) fn url(
    world: &mut World,
    path: String,
) -> anyhow::Result<Option<String>> {
    let url = world.resource(abs_asset_url());
    let base_url = &AbsAssetUrl::parse(url)?;
    let asset_url = AssetUrl::parse(path)?.resolve(&base_url)?;
    Ok(Some(asset_url.to_string()))
}