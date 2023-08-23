use ambient_native_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};
use anyhow::Context;

use crate::shared;

use super::{PackageCli, PackagePath};

pub struct BuildDirectories {
    /// The location where all built packages are stored. Used for the HTTP host.
    pub build_root_path: AbsAssetUrl,
    /// The location of the main package being executed. Used for everything else.
    pub main_package_path: AbsAssetUrl,
}
impl BuildDirectories {
    pub fn new_with_same_paths(path: AbsAssetUrl) -> Self {
        Self {
            build_root_path: path.clone(),
            main_package_path: path,
        }
    }
}

pub async fn build(
    package_cli: Option<&PackageCli>,
    package_path: PackagePath,
    assets: &AssetCache,
    release_build: bool,
) -> anyhow::Result<BuildDirectories> {
    let Some(package) = package_cli else {
        return Ok(BuildDirectories::new_with_same_paths(package_path.url.clone()));
    };

    if package.no_build {
        return Ok(BuildDirectories::new_with_same_paths(
            package_path.url.clone(),
        ));
    }

    let Some(package_fs_path) = package_path.fs_path else {
        return Ok(BuildDirectories::new_with_same_paths(package_path.url.clone()));
    };

    let build_path = package_fs_path.join("build");
    // The build step uses its own semantic to ensure that there is
    // no contamination, so that the built package can use its own
    // semantic based on the flat hierarchy.
    let mut semantic = ambient_package_semantic::Semantic::new().await?;
    let primary_package_scope_id =
        shared::package::add(None, &mut semantic, &package_path.url.join("ambient.toml")?).await?;

    let manifest = semantic
        .items
        .get(primary_package_scope_id)
        .manifest
        .clone();

    let build_config = ambient_build::BuildConfiguration {
        build_path: build_path.clone(),
        assets: assets.clone(),
        semantic: &mut semantic,
        optimize: release_build,
        clean_build: package.clean_build,
        build_wasm_only: package.build_wasm_only,
    };

    let package_name = manifest
        .package
        .name
        .as_deref()
        .unwrap_or_else(|| manifest.package.id.as_str());

    tracing::info!("Building package {:?}", package_name);

    let output_path = ambient_build::build(build_config, primary_package_scope_id)
        .await
        .context("Failed to build package")?;

    anyhow::Ok(BuildDirectories {
        build_root_path: AbsAssetUrl::from_file_path(build_path),
        main_package_path: AbsAssetUrl::from_file_path(output_path),
    })
}
