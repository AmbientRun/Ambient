use std::sync::Arc;

use ambient_animation::{AnimationClip, AnimationClipFromUrl};
use ambient_core::asset_cache;
use ambient_ecs::World;
use ambient_model::ModelFromUrl;
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt},
    asset_url::{AbsAssetUrl, AnimationAssetType, ParseError, TypedAssetUrl},
};
use anyhow::Context;

use crate::shared::wit;

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
            asset_url
        };
        Ok(asset_url.to_string())
    })
}

pub fn get_animation_asset_status(
    world: &mut World,
    clip_url: &str,
) -> anyhow::Result<wit::asset::AssetCacheStatus> {
    let assets = world.resource(asset_cache());
    match peek_loaded_clip(assets, clip_url) {
        Ok(Some(_clip)) => Ok(wit::asset::AssetCacheStatus::Ready),
        Ok(None) => Ok(wit::asset::AssetCacheStatus::NotLoaded),
        Err(err) => Ok(wit::asset::AssetCacheStatus::FailedLoading(format!(
            "{:?}",
            err
        ))),
    }
}

pub fn get_animation_asset_metadata(
    world: &mut World,
    clip_urls: &[String],
) -> anyhow::Result<Vec<wit::asset::AnimationAssetMetadata>> {
    let assets = world.resource(asset_cache());

    let mut result: Vec<wit::asset::AnimationAssetMetadata> = Vec::with_capacity(clip_urls.len());
    for clip_url in clip_urls {
        let (binders, duration, status) = match peek_loaded_clip(assets, clip_url) {
            Ok(Some(clip)) => {
                let binders: Vec<String> = clip
                    .tracks
                    .iter()
                    .map(|x| match &x.target {
                        ambient_animation::AnimationTarget::BinderId(binder) => binder.clone(),
                        ambient_animation::AnimationTarget::Entity(_entity) => String::default(),
                    })
                    .collect();

                let duration = clip.duration();
                (binders, duration, wit::asset::AssetCacheStatus::Ready)
            }
            Ok(None) => (Vec::default(), 0.0, wit::asset::AssetCacheStatus::NotLoaded),
            Err(err) => (
                Vec::default(),
                0.0,
                wit::asset::AssetCacheStatus::FailedLoading(format!("{:?}", err)),
            ),
        };
        result.push(wit::asset::AnimationAssetMetadata {
            binders,
            duration,
            status,
        });
    }

    Ok(result)
}

fn ok_wrap<R>(mut f: impl FnMut() -> R) -> anyhow::Result<R> {
    Ok(f())
}

fn parse_error_to_url_error(err: ParseError) -> wit::asset::UrlError {
    wit::asset::UrlError::InvalidUrl(err.to_string())
}

fn peek_loaded_clip(
    assets: &AssetCache,
    clip_url: &str,
) -> anyhow::Result<Option<Arc<AnimationClip>>> {
    let asset_url: TypedAssetUrl<AnimationAssetType> =
        TypedAssetUrl::parse(clip_url).context("Invalid clip url")?;
    let clip_asset_url: TypedAssetUrl<AnimationAssetType> = asset_url
        .abs()
        .context(format!("Expected absolute url, got: {}", clip_url))?
        .into();

    if let Some(asset) = ModelFromUrl(
        clip_asset_url
            .model_crate()
            .context("Invalid clip url")?
            .model(),
    )
    .peek(&assets)
    {
        let _model = asset.context("No such model")?;
    } else {
        return Ok(None);
    }

    if let Some(clip) = AnimationClipFromUrl::new(asset_url.unwrap_abs(), true).peek(assets) {
        Ok(Some(clip.context("No such clip")?))
    } else {
        Ok(None)
    }
}
