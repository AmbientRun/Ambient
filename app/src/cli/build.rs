use ambient_build::BuildConfiguration;
use ambient_native_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};
use anyhow::Context;

use crate::shared::package;

use super::PackageCli;

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
    package_cli: &PackageCli,
    assets: &AssetCache,
    release_build: bool,
    building_for_deploy: bool,
) -> anyhow::Result<BuildDirectories> {
    let package_path = package_cli.package_path()?;

    if package_cli.no_build {
        return Ok(BuildDirectories::new_with_same_paths(
            package_path.url.clone(),
        ));
    }

    let Some(package_fs_path) = package_path.fs_path else {
        return Ok(BuildDirectories::new_with_same_paths(package_path.url.clone()));
    };

    let root_build_path = package_fs_path.join("build");
    let main_manifest_url = package_path.url.join("ambient.toml")?;

    if package_cli.clean_build {
        tracing::debug!("Removing build directory: {root_build_path:?}");
        match tokio::fs::remove_dir_all(&root_build_path).await {
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err).context("Failed to remove build directory");
            }
        }
    }

    // Do a pre-build scan where we construct a semantic for the package
    // in non-deploy mode to gather the full graph of dependencies.
    // This is then used to walk the graph and build each package in
    // the correct order.
    //
    // Using a second semantic for this is quite excessive - it results in a lot of
    // unnecessary data churn - but refactoring the semantic to be able to
    // decouple the construction of the graph from the construction of the semantic
    // will require additional engineering.
    //
    // TODO: Decouple this properly, so that there is an async gather phase that can be
    // done independently from the rest of the semantic, which can be synchronous.
    let mut queue: Vec<_> = {
        let mut semantic = ambient_package_semantic::Semantic::new(false).await?;
        let primary_package_scope_id =
            package::add(None, &mut semantic, &main_manifest_url).await?;
        semantic
            .items
            .scope_and_dependencies(primary_package_scope_id)
            .into_iter()
            .flat_map(|id| semantic.items.get(id).source.to_local_url())
            .rev()
            .collect()
    };

    // The build step starts with a fresh semantic, and adds to it as more builds are done.
    // This allows for changes to be made to the packages between builds - most notably,
    // the ability to insert deployments as dependencies.
    let mut output_path = root_build_path.clone();
    let mut semantic = ambient_package_semantic::Semantic::new(building_for_deploy).await?;
    while let Some(package_url) = queue.pop() {
        let package_item_id =
            package::add(None, &mut semantic, &AbsAssetUrl(package_url.clone())).await?;
        let package_id = semantic.items.get(package_item_id).data.id.clone();

        let build_path = ambient_build::build_package(
            BuildConfiguration {
                assets: assets.clone(),
                semantic: &semantic,
                optimize: release_build,
                build_wasm_only: package_cli.build_wasm_only,
                building_for_deploy,
            },
            root_build_path.join(package_id.as_str()),
            package_item_id,
        )
        .await?;

        if package_url == main_manifest_url.0 {
            output_path = build_path;
        }
    }

    anyhow::Ok(BuildDirectories {
        build_root_path: AbsAssetUrl::from_file_path(root_build_path),
        main_package_path: AbsAssetUrl::from_file_path(output_path),
    })
}
