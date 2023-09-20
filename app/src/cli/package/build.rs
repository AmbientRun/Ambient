use std::{collections::HashSet, future::Future, path::PathBuf};

use ambient_build::BuildResult;
use ambient_native_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};
use ambient_package::BuildSettings;
use ambient_package_semantic::RetrievableFile;

use anyhow::Context;
use clap::Parser;

use super::PackageCli;

#[derive(Parser, Clone, Debug)]
/// Builds the package
pub struct Build {
    #[command(flatten)]
    pub package: PackageCli,
}

pub struct BuildDirectories {
    /// The location where all built packages are stored. Used for the HTTP host.
    pub build_root_path: AbsAssetUrl,
    /// The location of the main package being built. Used for everything else.
    pub main_package_path: AbsAssetUrl,
    /// The name of the main package being built.
    pub main_package_name: String,
}
impl BuildDirectories {
    pub fn new_with_same_paths(path: AbsAssetUrl) -> Self {
        Self {
            build_root_path: path.clone(),
            main_package_name: "Remote package".to_string(),
            main_package_path: path,
        }
    }
}

pub async fn handle(
    build: &Build,
    assets: &AssetCache,
    release_build: bool,
) -> anyhow::Result<BuildDirectories> {
    handle_inner(&build.package, assets, release_build).await
}

pub async fn handle_inner(
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
        return Ok(BuildDirectories::new_with_same_paths(
            main_package_path.url.clone(),
        ));
    };

    let build_wasm_only = package_cli.build_wasm_only;
    let clean_build = package_cli.clean_build;

    self::build(
        assets,
        main_package_fs_path,
        clean_build,
        false,
        release_build,
        build_wasm_only,
        HashSet::new(),
        |_| async { Ok(()) },
        |_, _, _| async { Ok(()) },
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
    deploy: bool,
    release: bool,
    wasm_only: bool,
    // Used by deploy to avoid building packages that have already been built
    // by other packages
    skip_building: HashSet<PathBuf>,
    mut pre_build: impl FnMut(PathBuf) -> PrebuildRet,
    mut post_build: impl FnMut(PathBuf, PathBuf, bool) -> PostbuildRet,
) -> anyhow::Result<BuildDirectories> {
    let root_build_path = main_package_fs_path.join("build");
    let main_manifest_url = AbsAssetUrl::from_file_path(main_package_fs_path.join("ambient.toml"));

    if clean_build {
        tracing::debug!("Removing build directory {root_build_path:?}");
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
        let primary_package_scope_id = semantic
            .add_package(RetrievableFile::Url(main_manifest_url.0.clone()), None)
            .await?;
        semantic
            .resolve_all()
            .context("Failed to resolve dependencies for pre-build")?;

        semantic
            .items
            .scope_and_dependencies(primary_package_scope_id)
            .into_iter()
            .flat_map(|id| semantic.items.get(id).source.as_local_path())
            .filter(|path| !skip_building.contains(path))
            .rev()
            .collect()
    };

    let settings = BuildSettings {
        release,
        wasm_only,
        deploy,
    };

    // For each package, build the package using a fresh semantic.
    // A fresh semantic is used to ensure that the package is being built with
    // the correct dependencies after they have been deployed (if necessary).
    let mut output_path = root_build_path.clone();
    let mut output_package_name = String::new();
    while let Some(manifest_path) = queue.pop() {
        pre_build(manifest_path.clone()).await?;

        let BuildResult {
            build_path,
            package_name,
            was_built,
        } = ambient_build::build_package(assets, &settings, &manifest_path, &root_build_path)
            .await?;

        post_build(manifest_path.clone(), build_path.clone(), was_built).await?;

        if AbsAssetUrl::from_file_path(manifest_path) == main_manifest_url {
            output_path = build_path;
            output_package_name = package_name;
        }
    }

    anyhow::Ok(BuildDirectories {
        build_root_path: AbsAssetUrl::from_file_path(root_build_path),
        main_package_path: AbsAssetUrl::from_file_path(output_path),
        main_package_name: output_package_name,
    })
}
