use std::future::Future;
use std::path::PathBuf;

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

pub async fn handle(
    package_cli: &PackageCli,
    assets: &AssetCache,
    release_build: bool,
) -> anyhow::Result<BuildDirectories> {
    let main_package_path = package_cli.package_path()?;

    if package_cli.no_build {
        return Ok(BuildDirectories::new_with_same_paths(
            main_package_path.url.clone(),
        ));
    }

    let Some(main_package_fs_path) = main_package_path.fs_path else {
        return Ok(BuildDirectories::new_with_same_paths(main_package_path.url.clone()));
    };

    let build_wasm_only = package_cli.build_wasm_only;
    let clean_build = package_cli.clean_build;

    build(
        assets,
        main_package_fs_path,
        clean_build,
        false,
        release_build,
        build_wasm_only,
        |_| async { Ok(()) },
        |_, _| async { Ok(()) },
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn build<
    PrebuildRet: Future<Output = anyhow::Result<()>>,
    PostbuildRet: Future<Output = anyhow::Result<()>>,
>(
    assets: &AssetCache,
    main_package_fs_path: PathBuf,
    clean_build: bool,
    building_for_deploy: bool,
    release_build: bool,
    build_wasm_only: bool,
    mut pre_build: impl FnMut(PathBuf) -> PrebuildRet,
    mut post_build: impl FnMut(PathBuf, PathBuf) -> PostbuildRet,
) -> anyhow::Result<BuildDirectories> {
    let root_build_path = main_package_fs_path.join("build");
    let main_manifest_url = AbsAssetUrl::from_file_path(main_package_fs_path.join("ambient.toml"));

    if clean_build {
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
    let mut queue: Vec<_> = {
        let mut semantic = ambient_package_semantic::Semantic::new(false).await?;
        let primary_package_scope_id =
            package::add(None, &mut semantic, &main_manifest_url).await?;
        semantic
            .items
            .scope_and_dependencies(primary_package_scope_id)
            .into_iter()
            .flat_map(|id| semantic.items.get(id).source.as_local_path())
            .rev()
            .collect()
    };

    // For each package, build the package using a fresh semantic.
    // A fresh semantic is used to ensure that the package is being built with
    // the correct dependencies after they have been deployed (if necessary).
    let mut output_path = root_build_path.clone();
    while let Some(package_path) = queue.pop() {
        pre_build(package_path.clone()).await?;

        let package_url = AbsAssetUrl::from_file_path(&package_path).0;

        let mut semantic = ambient_package_semantic::Semantic::new(building_for_deploy).await?;
        let package_item_id =
            package::add(None, &mut semantic, &AbsAssetUrl(package_url.clone())).await?;
        let package_id = semantic.items.get(package_item_id).data.id.clone();

        let build_path = ambient_build::build_package(
            BuildConfiguration {
                assets: assets.clone(),
                semantic: &semantic,
                optimize: release_build,
                build_wasm_only,
                building_for_deploy,
            },
            root_build_path.join(package_id.as_str()),
            package_item_id,
        )
        .await?;

        post_build(package_path.clone(), build_path.clone()).await?;

        if package_url == main_manifest_url.0 {
            output_path = build_path;
        }
    }

    anyhow::Ok(BuildDirectories {
        build_root_path: AbsAssetUrl::from_file_path(root_build_path),
        main_package_path: AbsAssetUrl::from_file_path(output_path),
    })
}
