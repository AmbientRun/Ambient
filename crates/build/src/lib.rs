use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use ambient_asset_cache::{AssetCache, SyncAssetKeyExt};
use ambient_native_std::{asset_url::AbsAssetUrl, AmbientVersion};
use ambient_package::{BuildMetadata, Manifest as PackageManifest, Version};
use ambient_package_semantic::{ItemId, Package, Semantic};
use ambient_std::path::path_to_unix_string_lossy;
use anyhow::Context;
use futures::FutureExt;
use itertools::Itertools;
use pipelines::{FileCollection, ProcessCtx, ProcessCtxKey};
use walkdir::WalkDir;

pub mod migrate;
pub mod pipelines;

pub struct BuildConfiguration<'a> {
    pub build_path: PathBuf,
    pub assets: AssetCache,
    pub semantic: &'a mut Semantic,
    pub optimize: bool,
    pub clean_build: bool,
    pub build_wasm_only: bool,
    /// If true, the build will be optimized for deployment.
    ///
    /// At this moment, this is only removing the local path dependencies from the manifest.
    pub building_for_deploy: bool,
}

pub async fn build(
    config: BuildConfiguration<'_>,
    root_package_id: ItemId<Package>,
) -> anyhow::Result<PathBuf> {
    let BuildConfiguration {
        build_path,
        assets,
        semantic,
        optimize,
        clean_build,
        build_wasm_only,
        building_for_deploy,
    } = config;

    if clean_build {
        tracing::debug!("Removing build directory: {build_path:?}");
        match tokio::fs::remove_dir_all(&build_path).await {
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err).context("Failed to remove build directory");
            }
        }
    }

    let mut queue = semantic.items.scope_and_dependencies(root_package_id);
    queue.reverse();

    let mut output_path = build_path.clone();
    while let Some(package_id) = queue.pop() {
        let id = {
            let package = semantic.items.get(package_id);
            if package.source.as_remote_url().is_some() {
                continue;
            }
            package.data.id.clone()
        };
        let build_path = build_path.join(id.as_str());
        build_package(
            BuildConfiguration {
                build_path: build_path.clone(),
                assets: assets.clone(),
                semantic,
                optimize,
                clean_build,
                build_wasm_only,
                building_for_deploy,
            },
            package_id,
        )
        .await?;

        if package_id == root_package_id {
            output_path = build_path;
        }
    }

    Ok(output_path)
}

/// This takes the path to an Ambient package and builds it. An Ambient package is expected to
/// have the following structure:
///
/// assets/**  Here assets such as .glb files are stored. Any files found in this directory will be processed
/// src/**  This is where you store Rust source files
/// build  This is the output directory, and is created when building
/// ambient.toml  This is a metadata file to describe the package
async fn build_package(
    config: BuildConfiguration<'_>,
    package_id: ItemId<Package>,
) -> anyhow::Result<()> {
    let BuildConfiguration {
        build_path,
        assets,
        semantic,
        optimize,
        build_wasm_only,
        building_for_deploy,
        ..
    } = config;

    let (mut manifest, package_path) = {
        let package = semantic.items.get(package_id);
        (
            package.manifest.clone(),
            package
                .source
                .as_path()
                .context("the package has no local path")?
                .parent()
                .context("the package path has no parent")?
                .to_owned(),
        )
    };

    let last_build_time = tokio::fs::read_to_string(build_path.join(BuildMetadata::FILENAME))
        .await
        .ok()
        .map(|c| BuildMetadata::parse(&c))
        .transpose()?
        .and_then(|md| Some(chrono::DateTime::parse_from_rfc3339(&md.last_build_time?)))
        .transpose()?
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // This is only used to filter out the build files that were produced by building
    // this package individually, if they exist. This is to avoid marking this package
    // as dirty in a whole-workspace build if it was merely built individually.
    let package_individual_build_path = package_path.join("build");
    // NOTE: This logic is not optimal. It *will* consider files that shouldn't be considered
    // (like a top-level target folder, or the individual build folders of subdirectories)
    // but it's good enough for now. In future, we may want to consider using `ignore`
    // to filter out files that shouldn't be considered.
    let last_modified_time = get_files_in_path(&package_path)
        .filter(|p| !p.starts_with(&build_path) && !p.starts_with(&package_individual_build_path))
        .filter_map(|f| f.metadata().ok()?.modified().ok())
        .map(chrono::DateTime::<chrono::Utc>::from)
        .max();

    let name = manifest
        .package
        .name
        .as_deref()
        .unwrap_or_else(|| manifest.package.id.as_str());

    if last_build_time
        .zip(last_modified_time)
        .is_some_and(|(build, modified)| modified < build)
    {
        tracing::info!(
            "Skipping unmodified package \"{name}\" ({})",
            manifest.package.id
        );
        return Ok(());
    }

    tracing::info!("Building package \"{name}\" ({})", manifest.package.id);

    let assets_path = package_path.join("assets");
    tokio::fs::create_dir_all(&build_path)
        .await
        .context("Failed to create build directory")?;

    if !build_wasm_only {
        build_assets(&assets, &assets_path, &build_path, false).await?;
    }

    build_rust_if_available(&package_path, &manifest, &build_path, optimize)
        .await
        .with_context(|| format!("Failed to build Rust {build_path:?}"))?;

    // Bodge: for local builds, rewrite the dependencies to be relative to this package,
    // assuming that they are all in the same folder
    {
        let package = semantic.items.get(package_id);
        let alias_to_dependency = package
            .dependencies
            .iter()
            .map(|(id, dep)| anyhow::Ok((id.clone(), semantic.items.get(dep.id).data.id.clone())))
            .collect::<Result<HashMap<_, _>, _>>()?;

        for (alias, dependency) in manifest.dependencies.iter_mut() {
            let ambient_package::Dependency { path, .. } = dependency;

            if let Some(original_dependency_name) = alias_to_dependency.get(alias) {
                let new_path = Path::new("..").join(original_dependency_name.as_str());
                if ambient_std::path::normalize(&build_path.join(&new_path)).exists() {
                    // Only update the path if the directory actually exists. This prevents us from
                    // accidentally setting the path to a dependency that doesn't exist locally.
                    *path = Some(new_path);
                }
            }

            // If we are building for deployment, remove all local path dependencies
            if building_for_deploy {
                *path = None;
            }
        }
    }

    store_manifest(&manifest, &build_path).await?;
    store_metadata(&build_path).await?;

    Ok(())
}

fn get_files_in_path(path: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|x| x.is_file()).unwrap_or(false))
        .map(|x| x.into_path())
}

pub async fn build_assets(
    assets: &AssetCache,
    assets_path: &Path,
    build_path: &Path,
    for_import_only: bool,
) -> anyhow::Result<()> {
    let files = get_files_in_path(assets_path).map(Into::into).collect_vec();

    let has_errored = Arc::new(AtomicBool::new(false));

    let anim_files = Arc::new(parking_lot::Mutex::new(vec![]));
    let anim_files_clone = anim_files.clone();

    let ctx = ProcessCtx {
        assets: assets.clone(),
        files: FileCollection(Arc::new(files)),
        in_root: AbsAssetUrl::from_directory_path(assets_path),
        out_root: AbsAssetUrl::from_directory_path(build_path.join("assets")),
        input_file_filter: None,
        package_name: "".to_string(),
        write_file: Arc::new({
            let build_path = build_path.to_owned();
            move |path, contents| {
                let path = build_path.join("assets").join(path);

                if for_import_only {
                    if let Some(ext) = path.extension() {
                        if ext == "anim" {
                            if !anim_files_clone.lock().contains(&path) {
                                anim_files_clone.lock().push(path.clone());
                            } else {
                                println!("🤔 repeated importing; please check the pipeline.toml");
                            }
                        }
                    }
                }

                async move {
                    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                    tokio::fs::write(&path, contents).await.unwrap();
                    AbsAssetUrl::from_file_path(path)
                }
                .boxed()
            }
        }),
        on_status: Arc::new(|msg| {
            log::debug!("{}", msg);
            async {}.boxed()
        }),
        on_error: Arc::new({
            let has_errored = has_errored.clone();
            move |err| {
                log::error!("{:?}", err);
                has_errored.store(true, Ordering::SeqCst);
                async {}.boxed()
            }
        }),
    };

    ProcessCtxKey.insert(&ctx.assets, ctx.clone());

    pipelines::process_pipelines(&ctx)
        .await
        .with_context(|| format!("Failed to process pipelines for {assets_path:?}"))?;

    if !anim_files.lock().is_empty() {
        println!("🐆 Available animation files: {:?}", anim_files.lock());
        println!("🧂 You can use the animation files like this:");
        println!("```rust");
        for path in anim_files.lock().iter() {
            println!(
                "PlayClipFromUrlNode::new(assets::url({}));",
                format!("{:?}", path).replace("build/assets/", "")
            );
        }
        println!("```");
        println!("📘 Learn more about animations:");
        println!("🔗 https://ambientrun.github.io/Ambient/reference/animations.html");
    }
    if has_errored.load(Ordering::SeqCst) {
        anyhow::bail!("Failed to build assets");
    }

    Ok(())
}

pub async fn build_rust_if_available(
    package_path: &Path,
    manifest: &PackageManifest,
    build_path: &Path,
    optimize: bool,
) -> anyhow::Result<()> {
    let cargo_toml_path = package_path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(());
    }

    let rustc = ambient_rustc::Rust::get_system_installation().await?;

    for feature in &manifest.build.rust.feature_multibuild {
        for (path, bytecode) in rustc.build(package_path, optimize, &[feature])? {
            let component_bytecode = ambient_wasm::shared::build::componentize(&bytecode)?;

            let output_path = build_path.join(feature);
            std::fs::create_dir_all(&output_path)?;

            let filename = path.file_name().context("no filename")?;
            tokio::fs::write(output_path.join(filename), component_bytecode).await?;
        }
    }

    Ok(())
}

fn get_component_paths(target: &str, build_path: &Path) -> Vec<String> {
    std::fs::read_dir(build_path.join(target))
        .ok()
        .map(|rd| {
            rd.filter_map(Result::ok)
                .map(|p| p.path())
                .filter(|p| p.extension().unwrap_or_default() == "wasm")
                .map(|p| path_to_unix_string_lossy(p.strip_prefix(build_path).unwrap()))
                .collect()
        })
        .unwrap_or_default()
}

async fn store_manifest(manifest: &PackageManifest, build_path: &Path) -> anyhow::Result<()> {
    let manifest_path = build_path.join("ambient.toml");
    tokio::fs::write(&manifest_path, toml::to_string(&manifest)?).await?;
    Ok(())
}

async fn store_metadata(build_path: &Path) -> anyhow::Result<BuildMetadata> {
    let AmbientVersion { version, revision } = AmbientVersion::default();
    let metadata = BuildMetadata {
        ambient_version: Version::new_from_str(&version).expect("Failed to parse version"),
        ambient_revision: revision,
        client_component_paths: get_component_paths("client", build_path),
        server_component_paths: get_component_paths("server", build_path),
        last_build_time: Some(chrono::Utc::now().to_rfc3339()),
    };
    let metadata_path = build_path.join(BuildMetadata::FILENAME);
    tokio::fs::write(&metadata_path, toml::to_string(&metadata)?).await?;
    Ok(metadata)
}
