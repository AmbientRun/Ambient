use std::{
    path::{Path, PathBuf}, sync::Arc
};

use elements_asset_cache::{AssetCache, SyncAssetKeyExt};
use elements_physics::physx::{Physics, PhysicsKey};
use elements_project::Manifest as ProjectManifest;
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
/// elements.toml  This is a metadata file to describe the project
pub async fn build(physics: Physics, _assets: &AssetCache, path: PathBuf, manifest: &ProjectManifest) {
    elements_ecs::ComponentRegistry::get_mut().add_external_from_iterator(manifest.all_defined_components(false).unwrap().into_iter());

    let target_path = path.join("target");
    let assets_path = path.join("assets");
    build_assets(physics, &assets_path, &target_path).await;
    build_scripts(&path, manifest, &target_path).await.unwrap();
}

async fn build_assets(physics: Physics, assets_path: &Path, target_path: &Path) {
    let files = WalkDir::new(assets_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|x| x.is_file()).unwrap_or(false))
        .map(|x| AbsAssetUrl::from_file_path(x.into_path()))
        .collect_vec();
    let assets = AssetCache::new_with_config(tokio::runtime::Handle::current(), None);
    PhysicsKey.insert(&assets, physics);
    let ctx = ProcessCtx {
        assets: assets.clone(),
        files: Arc::new(files),
        in_root: AbsAssetUrl::from_directory_path(assets_path),
        out_root: AbsAssetUrl::from_directory_path(target_path.join("assets")),
        input_file_filter: None,
        package_name: "".to_string(),
        write_file: Arc::new({
            let target_path = target_path.to_owned();
            move |path, contents| {
                let path = target_path.join("assets").join(path);
                async move {
                    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                    tokio::fs::write(&path, contents).await.unwrap();
                    AbsAssetUrl::from_file_path(path)
                }
                .boxed()
            }
        }),
        on_status: Arc::new(|msg| {
            log::info!("{}", msg);
            async {}.boxed()
        }),
        on_error: Arc::new(|err| {
            log::error!("{:?}", err);
            async {}.boxed()
        }),
    };
    ProcessCtxKey.insert(&ctx.assets, ctx.clone());
    pipelines::process_pipelines(&ctx).await;
}

async fn build_scripts(path: &Path, manifest: &ProjectManifest, target_path: &Path) -> anyhow::Result<()> {
    let cargo_toml_path = path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(());
    }

    let toml = cargo_toml::Manifest::from_str(&tokio::fs::read_to_string(&cargo_toml_path).await?)?;
    match toml.package {
        Some(package) if package.name == manifest.project.id.as_ref() => {}
        Some(package) => {
            anyhow::bail!(
                "The name of the package in the Cargo.toml ({}) does not match the project's ID ({})",
                package.name,
                manifest.project.id
            );
        }
        None => anyhow::bail!("No [package] present in Cargo.toml for project {}", manifest.project.id.as_ref()),
    }

    let rust_path = elements_std::path::normalize(&std::env::current_dir()?.join("rust"));
    let rustc = elements_rustc::Rust::install_or_get(&rust_path).await?;
    let bytecode = rustc.build(path, manifest.project.id.as_ref())?;

    tokio::fs::write(target_path.join(format!("{}.wasm", manifest.project.id)), bytecode).await?;

    Ok(())
}
