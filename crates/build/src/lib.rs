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
use ambient_package::{
    BuildMetadata, BuildMetadataError, BuildSettings, Manifest as PackageManifest,
};
use ambient_package_semantic::{package_dependency_to_retrievable_file, RetrievableFile, Semantic};
use ambient_package_semantic_native::add_to_semantic_and_register_components;
use ambient_shared_types::asset::BuildAsset;
use ambient_std::path::path_to_unix_string_lossy;
use anyhow::Context;
use futures::FutureExt;
use itertools::Itertools;
use pipelines::{out_asset::OutAsset, FileCollection, ProcessCtx, ProcessCtxKey};
use walkdir::WalkDir;

pub mod migrate;
pub mod pipelines;

#[derive(Clone, Debug)]
pub struct BuildResult {
    pub build_path: PathBuf,
    pub package_name: String,
    pub was_built: bool,
}

/// This takes the path to a single Ambient package and builds it.
/// It assumes all of its dependencies are already built.
///
/// An Ambient package is expected to have the following structure:
///
/// assets/**  Here assets such as .glb files are stored. Any files found in this directory will be processed
/// src/**  This is where you store Rust source files
/// build  This is the output directory, and is created when building
/// ambient.toml  This is a metadata file to describe the package
pub async fn build_package(
    assets: &AssetCache,
    settings: &BuildSettings,
    package_path: &Path,
    root_build_path: &Path,
) -> anyhow::Result<BuildResult> {
    let mut semantic = Semantic::new(settings.deploy).await?;

    let package_item_id = add_to_semantic_and_register_components(
        &mut semantic,
        &AbsAssetUrl(AbsAssetUrl::from_file_path(package_path).0),
    )
    .await?;

    let package_id = semantic.items.get(package_item_id).data.id.clone();
    let build_path = root_build_path.join(package_id.as_str());
    let output_manifest_path = build_path.join("ambient.toml");

    let (mut manifest, package_path) = {
        let package = semantic.items.get(package_item_id);
        (
            package.manifest.clone(),
            package
                .source
                .as_local_path()
                .context("the package has no local path")?
                .parent()
                .context("the package path has no parent")?
                .to_owned(),
        )
    };
    let package_name = &manifest.package.name;

    // Bodge: for local builds, rewrite the dependencies to be relative to this package,
    // assuming that they are all in the same folder
    {
        let package = semantic.items.get(package_item_id);
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
            if settings.deploy {
                *path = None;
            }
        }
    }

    let build_metadata = get_build_metadata(RetrievableFile::Path(
        build_path.join(BuildMetadata::FILENAME),
    ))
    .await?;
    let (last_build_time, last_build_settings) = match build_metadata {
        Some(md) => (md.last_build_time()?, Some(md.settings)),
        None => (None, None),
    };

    // Get the last build time of all dependencies to determine if this package needs to be rebuilt
    let dependency_max_last_build_times = {
        async fn dependency_to_build_time(
            omr: &RetrievableFile,
            deploy: bool,
            dependency: &ambient_package::Dependency,
        ) -> anyhow::Result<Option<chrono::DateTime<chrono::Utc>>> {
            let metadata_path = package_dependency_to_retrievable_file(omr, deploy, dependency)?
                .map(|p| p.parent_join(Path::new(BuildMetadata::FILENAME)))
                .transpose()?;

            Ok(match metadata_path {
                Some(metadata_path) => get_build_metadata(metadata_path)
                    .await?
                    .and_then(|md| md.last_build_time().transpose())
                    .transpose()?,
                None => None,
            })
        }

        let output_manifest_retrievable = RetrievableFile::Path(output_manifest_path.clone());
        futures::future::try_join_all(manifest.dependencies.values().map(|dep| {
            dependency_to_build_time(&output_manifest_retrievable, settings.deploy, dep)
        }))
        .await?
        .into_iter()
        .flatten()
        .max()
    };

    // This is only used to filter out the build files that were produced by building
    // this package individually, if they exist. This is to avoid marking this package
    // as dirty in a whole-workspace build if it was merely built individually.
    let package_individual_build_path = package_path.join("build");
    // NOTE: This logic is not optimal. It *will* consider files that shouldn't be considered
    // (like a top-level target folder, or the individual build folders of subdirectories)
    // but it's good enough for now. In future, we may want to consider using `ignore`
    // to filter out files that shouldn't be considered.
    //
    // Additionally, it includes the build time of each dependency to ensure that this package
    // gets rebuilt if its dependencies have been updated.
    let last_modified_time = get_files_in_path(&package_path)
        .filter(|p| !p.starts_with(&build_path) && !p.starts_with(&package_individual_build_path))
        .filter_map(|f| f.metadata().ok()?.modified().ok())
        .map(chrono::DateTime::<chrono::Utc>::from)
        .chain(dependency_max_last_build_times.into_iter())
        .max();

    let last_modified_before_build = last_build_time
        .zip(last_modified_time)
        .is_some_and(|(build, modified)| modified < build);

    if last_build_settings.as_ref() == Some(settings) && last_modified_before_build {
        tracing::info!(
            "Skipping unmodified package \"{package_name}\" ({})",
            manifest.package.id
        );
        return Ok(BuildResult {
            build_path,
            package_name: package_name.clone(),
            was_built: false,
        });
    }

    tracing::info!(
        "Building package \"{package_name}\" ({})",
        manifest.package.id
    );

    let assets_path = package_path.join("assets");
    tokio::fs::create_dir_all(&build_path)
        .await
        .context("Failed to create build directory")?;

    let assets = if !settings.wasm_only {
        build_assets(assets, &assets_path, &build_path, false).await?
    } else {
        vec![]
    };

    build_rust_if_available(&package_path, &manifest, &build_path, settings.release)
        .await
        .with_context(|| format!("Failed to build Rust in {build_path:?}"))?;

    tokio::fs::write(&output_manifest_path, toml::to_string(&manifest)?).await?;

    store_metadata(&package_path, &build_path, settings, &assets).await?;

    Ok(BuildResult {
        build_path,
        package_name: package_name.clone(),
        was_built: true,
    })
}

async fn get_build_metadata(
    metadata: RetrievableFile,
) -> Result<Option<BuildMetadata>, BuildMetadataError> {
    metadata
        .get()
        .await
        .ok()
        .map(|c| BuildMetadata::parse(&c))
        .transpose()
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
) -> anyhow::Result<Vec<OutAsset>> {
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

    let assets = pipelines::process_pipelines(&ctx)
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

    Ok(assets)
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
        let output_path = build_path.join(feature);
        // We can safely remove this directory and its contents as Rust/Cargo maintains
        // its own build cache. Removing this directory should prevent stale files from
        // being used.
        let _ = std::fs::remove_dir_all(&output_path);
        std::fs::create_dir_all(&output_path)?;

        for (path, bytecode) in rustc.build(package_path, optimize, &[feature])? {
            let component_bytecode = ambient_wasm::shared::build::componentize(&bytecode)?;
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

async fn store_metadata(
    package_path: &Path,
    build_path: &Path,
    settings: &BuildSettings,
    assets: &[OutAsset],
) -> anyhow::Result<BuildMetadata> {
    fn strip_path(path: PathBuf, prefix: &Path) -> PathBuf {
        path.strip_prefix(prefix)
            .map(|p| p.to_owned())
            .unwrap_or(path)
    }

    let AmbientVersion { version, revision } = AmbientVersion::default();
    let metadata = BuildMetadata {
        ambient_version: version,
        ambient_revision: revision,
        client_component_paths: get_component_paths("client", build_path),
        server_component_paths: get_component_paths("server", build_path),
        last_build_time: Some(chrono::Utc::now().to_rfc3339()),
        settings: settings.clone(),
        asset: assets
            .iter()
            .flat_map(|a| {
                Some(BuildAsset {
                    type_: a.type_,
                    input: a
                        .source
                        .as_ref()
                        .and_then(|s| s.to_file_path().ok().flatten())
                        .map(|p| strip_path(p, package_path)),
                    output: strip_path(a.content.as_content()?.to_file_path().ok()??, build_path),
                })
            })
            .collect(),
    };
    let metadata_path = build_path.join(BuildMetadata::FILENAME);
    tokio::fs::write(&metadata_path, toml::to_string(&metadata)?).await?;
    Ok(metadata)
}
