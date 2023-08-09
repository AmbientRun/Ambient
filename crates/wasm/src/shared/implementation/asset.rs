use std::path::Path;

use ambient_core::asset_cache;
use ambient_ecs::World;
use ambient_native_std::asset_url::ParseError;
use ambient_project::SnakeCaseIdentifier;
use anyhow::Context;

use crate::shared::wit;

pub(crate) fn url(
    world: &World,
    ember_id: String,
    path: String,
    resolve: bool,
) -> anyhow::Result<Result<String, wit::asset::UrlError>> {
    let assets = world.resource(asset_cache()).clone();
    let semantic = ambient_ember_semantic_native::world_semantic(world);
    let semantic = semantic.lock().unwrap();
    let scope_id = semantic
        .get_scope_id_by_name(
            &SnakeCaseIdentifier::new(ember_id.as_str())
                .map_err(|err| anyhow::anyhow!("failed to parse ember name '{ember_id}': {err}"))?,
        )
        .context("failed to find ember by specified name")?;

    let asset_url = ambient_ember_semantic_native::file_path(
        &semantic,
        scope_id,
        &Path::new("assets").join(&path),
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
