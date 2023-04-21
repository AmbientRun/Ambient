use ambient_core::asset_cache;
use ambient_ecs::World;
use ambient_std::asset_url::{AbsAssetUrl, ParseError};

use crate::shared::wit;

fn ok_wrap<R>(mut f: impl FnMut() -> R) -> anyhow::Result<R> {
    Ok(f())
}

fn parse_error_to_url_error(err: ParseError) -> wit::asset::UrlError {
    wit::asset::UrlError::InvalidUrl(err.to_string())
}

pub(crate) fn url(
    world: &World,
    path: String,
    resolve: bool,
) -> anyhow::Result<Result<String, wit::asset::UrlError>> {
    ok_wrap(move || {
        let assets = world.resource(asset_cache()).clone();
        let asset_url = AbsAssetUrl::from_asset_key(&path).map_err(parse_error_to_url_error)?;
        let asset_url = if resolve {
            asset_url
                .to_download_url(&assets)
                .map_err(parse_error_to_url_error)?
        } else {
            asset_url.0
        };
        Ok(asset_url.to_string())
    })
}
