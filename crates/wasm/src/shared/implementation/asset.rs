use ambient_std::{
    asset_url::{AbsAssetUrl, AssetUrl}
};

use ambient_network::{ServerWorldExt, content_base_url};
use ambient_ecs::World;

pub(crate) fn url(
    world: &World,
    path: String,
) -> anyhow::Result<Option<String>> {
    let url = world.synced_resource(content_base_url()).unwrap();
    let base_url = &AbsAssetUrl::parse(url)?;
    let asset_url = AssetUrl::parse(path)?.resolve(&base_url)?;
    Ok(Some(asset_url.to_string()))
}