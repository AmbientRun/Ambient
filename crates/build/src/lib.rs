use std::{path::PathBuf, sync::Arc};

use elements_asset_cache::AssetCache;
use elements_model_import::{model_crate::ModelCrate, MODEL_EXTENSIONS};
use elements_std::asset_url::AbsAssetUrl;
use futures::FutureExt;
use walkdir::WalkDir;

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
    build_assets(assets, assets_path, target_path).await;
}

async fn build_assets(assets: &AssetCache, assets_path: PathBuf, target_path: PathBuf) {
    for model_path in WalkDir::new(&assets_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|x| x.into_path())
        .filter(|e| MODEL_EXTENSIONS.iter().any(|x| x == &e.extension().unwrap_or_default().to_str().unwrap().to_lowercase()))
    {
        println!("model: {:?} {:?}", model_path, model_path.extension());
        let mut model = ModelCrate::new();
        model
            .import(assets, &AbsAssetUrl::from_file_path(&model_path), true, false, Arc::new(|path| async move { None }.boxed()))
            .await
            .unwrap();
        model.update_node_primitive_aabbs_from_cpu_meshes();
        model.model_mut().update_model_aabb();
        model.write_to_fs(&target_path.join("assets").join(&model_path.strip_prefix(&assets_path).unwrap())).await;
    }
}
