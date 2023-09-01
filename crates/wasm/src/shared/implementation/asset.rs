use std::path::Path;

use ambient_core::asset_cache;
use ambient_ecs::World;
use ambient_native_std::asset_url::ParseError;

use crate::shared::{conversion::FromBindgen, wit};

pub(crate) fn url(
    world: &World,
    package_id: wit::types::EntityId,
    path: String,
    resolve: bool,
) -> anyhow::Result<Result<String, wit::asset::UrlError>> {
    let assets = world.resource(asset_cache()).clone();

    let asset_url = ambient_package_semantic_native::file_path(
        world,
        package_id.from_bindgen(),
        &Path::new("assets").join(path),
    )?;

    ok_wrap(move || {
        Ok(if resolve {
            asset_url
                .to_download_url(&assets)
                .map_err(parse_error_to_url_error)?
                .to_string()
        } else {
            asset_url.to_string()
        })
    })
}

fn ok_wrap<R>(mut f: impl FnMut() -> R) -> anyhow::Result<R> {
    Ok(f())
}

fn parse_error_to_url_error(err: ParseError) -> wit::asset::UrlError {
    wit::asset::UrlError::InvalidUrl(err.to_string())
}
