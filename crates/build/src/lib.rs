use std::{path::PathBuf, sync::Arc};

use elements_asset_cache::{AssetCache, SyncAssetKeyExt};
use elements_model_import::{model_crate::ModelCrate, MODEL_EXTENSIONS};
use elements_std::asset_url::AbsAssetUrl;
use futures::FutureExt;
use itertools::Itertools;
use pipelines::{ProcessCtx, ProcessCtxKey};
use walkdir::WalkDir;

pub mod pipelines;

/// This takes the path to an Elements project and builds it. An Elements project is expected to
/// have the following structure:
///
/// assets/**  Here assets such as .glb files are stored. Any files found in this directory will be processed
/// src/**  This is where you store Rust source files
/// target  This is the output directory, and is created when building
/// Elements.toml  This is a metadata file to describe the project
pub async fn build(assets: &AssetCache, path: PathBuf) {
    let target_path = path.join("target");
    let assets_path = path.join("assets");
    build_assets(assets_path, target_path).await;
}

async fn build_assets(assets_path: PathBuf, target_path: PathBuf) {
    let files =
        WalkDir::new(&assets_path).into_iter().filter_map(|e| e.ok()).map(|x| AbsAssetUrl::from_file_path(x.into_path())).collect_vec();
    let assets = AssetCache::new_with_config(tokio::runtime::Handle::current(), None);
    let ctx = ProcessCtx {
        assets: assets.clone(),
        files: Arc::new(files),
        in_root: AbsAssetUrl::from_directory_path(&assets_path),
        out_root: AbsAssetUrl::from_directory_path(target_path.join("assets")),
        input_file_filter: None,
        package_name: "".to_string(),
        write_file: Arc::new(move |path, contents| {
            let path = target_path.join("assets").join(path);
            async move {
                std::fs::create_dir_all(path.parent().unwrap());
                tokio::fs::write(&path, contents).await.unwrap();
                AbsAssetUrl::from_file_path(path)
            }
            .boxed()
        }),
        on_status: Arc::new(|msg| {
            log::info!("{}", msg);
            async { () }.boxed()
        }),
        on_error: Arc::new(|err| {
            log::error!("{:?}", err);
            async { () }.boxed()
        }),
    };
    ProcessCtxKey.insert(&ctx.assets, ctx.clone());
    pipelines::process_pipelines(&ctx).await;
}
