use std::{collections::HashMap, path::PathBuf, sync::Arc};

use ambient_native_std::asset_cache::AssetCache;
use anyhow::Context;
use parking_lot::Mutex;

use crate::cli::build;

use super::{PackageCli, PackagePath};

#[allow(clippy::too_many_arguments)]
pub async fn handle(
    package: &PackageCli,
    extra_packages: &[PathBuf],
    assets: &AssetCache,
    token: &str,
    api_server: &str,
    release_build: bool,
    force_upload: bool,
    ensure_running: bool,
    context: &str,
) -> anyhow::Result<()> {
    if !release_build {
        // Using string interpolation due to a rustfmt bug where it will break
        // if any one line is too long
        log::warn!(
            "{} {}",
            "Deploying a debug build which might involve uploading large files.",
            "Remove `--debug` to deploy a release build."
        );
    }

    #[derive(Debug, Clone)]
    enum Deployment {
        Skipped,
        Deployed(String),
    }
    impl Deployment {
        fn as_deployed(&self) -> Option<String> {
            match self {
                Self::Deployed(v) => Some(v.clone()),
                _ => None,
            }
        }
    }

    let manifest_path_to_deployment_id =
        Arc::new(Mutex::new(HashMap::<PathBuf, Deployment>::new()));

    let Some(main_package_fs_path) = package.package_path()?.fs_path else {
        anyhow::bail!("Can only deploy a local package");
    };

    let all_package_paths = {
        let mut all_packages = vec![main_package_fs_path];
        all_packages.extend(extra_packages.iter().map(|p| {
            PackagePath::new_local(p)
                .expect("failed to construct local package path")
                .fs_path
                .expect("failed to get fs path for local package")
        }));
        all_packages
    };

    let mut first_deployment_id = None;
    for package_path in all_package_paths {
        let skip_building = manifest_path_to_deployment_id
            .lock()
            .keys()
            .cloned()
            .collect();

        let result = build::build(
            assets,
            package_path.clone(),
            package.clean_build,
            true,
            release_build,
            package.build_wasm_only,
            skip_building,
            |package_manifest_path| {
                // Before the build, rewrite all known dependencies to use their deployed version
                // if available.
                let manifest_path_to_deployment_id = manifest_path_to_deployment_id.clone();
                async move {
                    let package_path = package_manifest_path.parent().unwrap();

                    let mut manifest: toml_edit::Document =
                        tokio::fs::read_to_string(&package_manifest_path)
                            .await?
                            .parse()?;

                    let Some(dependencies) = manifest.as_table_mut().get_mut("dependencies") else {
                        return Ok(());
                    };

                    for (_, dependency) in dependencies.as_table_like_mut().unwrap().iter_mut() {
                        let Some(dependency) = dependency.as_table_like_mut() else {
                            continue;
                        };
                        let Some(dependency_path) = dependency.get("path").and_then(|i| i.as_str())
                        else {
                            continue;
                        };

                        let dependency_manifest_path = ambient_std::path::normalize(
                            &package_path.join(dependency_path).join("ambient.toml"),
                        );

                        if let Some(deployment_id) = manifest_path_to_deployment_id
                            .lock()
                            .get(&dependency_manifest_path)
                            .cloned()
                            .and_then(|d| d.as_deployed())
                        {
                            dependency.insert("deployment", toml_edit::value(deployment_id));
                        }
                    }

                    tokio::fs::write(&package_manifest_path, manifest.to_string()).await?;

                    Ok(())
                }
            },
            |package_manifest_path, build_path, was_built| {
                // After build, deploy the package.
                let manifest_path_to_deployment_id = manifest_path_to_deployment_id.clone();
                async move {
                    let deployment = if was_built {
                        let deployment_id =
                            ambient_deploy::deploy(api_server, token, build_path, force_upload)
                                .await?;
                        Deployment::Deployed(deployment_id)
                    } else {
                        // TODO: this check does not actually save much, as the process of deploying
                        // the package and updating the manifest invalidates the last-build-time check
                        // anyway. This means that it only really works for "root" packages, and not
                        // anything with dependencies.
                        //
                        // Consider either using another metric, or implementing a more intelligent
                        // algorithm.
                        Deployment::Skipped
                    };

                    manifest_path_to_deployment_id
                        .lock()
                        .insert(package_manifest_path.to_owned(), deployment);

                    Ok(())
                }
            },
        )
        .await?;

        let main_package_name = result.main_package_name;

        let deployment_id = manifest_path_to_deployment_id
            .lock()
            .get(&package_path.join("ambient.toml"))
            .cloned()
            .context("main package was not processed")?;

        match deployment_id {
            Deployment::Skipped => {
                log::info!(
                    "Package \"{main_package_name}\" was already deployed, skipping deployment"
                );
            }
            Deployment::Deployed(deployment_id) => {
                log::info!("Package \"{main_package_name}\" deployed successfully!");
                log::info!("  Deployment ID: {deployment_id}");
                log::info!("  Deploy URL: https://assets.ambient.run/{deployment_id}");

                if first_deployment_id.is_none() {
                    first_deployment_id = Some(deployment_id);
                }
            }
        }
    }

    if let Some(deployment_id) = first_deployment_id.filter(|_| ensure_running) {
        let spec = ambient_cloud_client::ServerSpec::new_with_deployment(deployment_id)
            .with_context(context.to_string());
        let server =
            ambient_cloud_client::ensure_server_running(assets, api_server, token.into(), spec)
                .await?;
        log::info!("Deployed package is running at {}", server.host);
    }

    Ok(())
}
