use std::{collections::HashMap, path::PathBuf, sync::Arc};

use ambient_native_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use ambient_settings::SettingsKey;
use anyhow::Context;
use clap::Parser;
use parking_lot::Mutex;

use crate::cli::{package::build, PackagePath};

use super::PackageArgs;
use colored::Colorize;

#[derive(Parser, Clone, Debug)]
#[command(disable_version_flag = true)]
/// Deploys the package
pub struct Deploy {
    #[command(flatten)]
    pub package: PackageArgs,
    /// Additional packages to deploy; this allows you to share deployed dependencies
    /// between packages when doing a group deploy
    #[arg(long)]
    pub extra_packages: Vec<PathBuf>,
    /// API server endpoint
    #[arg(long, default_value = ambient_shared_types::urls::API_URL)]
    pub api_server: String,
    /// Authentication token
    #[arg(short, long)]
    pub token: Option<String>,
    /// Don't use differential upload and upload all assets
    #[arg(long)]
    pub force_upload: bool,
    /// Ensure the package is running after deploying
    #[arg(long)]
    pub ensure_running: bool,
    /// Context to run the package in
    #[arg(long, requires("ensure_running"), default_value = "")]
    pub context: String,
    /// When supplied, updates the versions of all packages that will be deployed.
    /// It is very likely that you will need to use this to deploy a tree of packages.
    #[arg(long)]
    pub version: Option<String>,
}

pub async fn handle(args: &Deploy, assets: &AssetCache, release_build: bool) -> anyhow::Result<()> {
    let Deploy {
        package,
        extra_packages,
        api_server,
        token,
        force_upload,
        ensure_running,
        context,
        version,
    } = args;

    if !release_build {
        // Using string interpolation due to a rustfmt bug where it will break
        // if any one line is too long
        tracing::warn!(
            "{} {}",
            "Deploying a debug build which might involve uploading large files.",
            "Remove `--debug` to deploy a release build."
        );
    }

    let settings = SettingsKey.get(assets);
    let token = match token {
        Some(token) => token,
        None => {
            settings
                .general
                .api_token
                .as_ref()
                .with_context(|| format!(
                    "No API token provided.\n\nYou can provide one with `--token` or by using `ambient login`.\n\nSign up for an account on the Ambient website, then go to your profile page to generate an API token: '{}'.",
                    ambient_shared_types::urls::AMBIENT_WEB_APP_URL
                ))?
        }
    };

    #[derive(Debug, Clone)]
    enum Deployment {
        Skipped,
        Deployed { package_id: String, version: String },
    }
    impl Deployment {
        fn as_deployed(&self) -> Option<(String, String)> {
            match self {
                Self::Deployed {
                    package_id,
                    version,
                } => Some((package_id.clone(), version.clone())),
                _ => None,
            }
        }
    }

    let manifest_path_to_version = Arc::new(Mutex::new(HashMap::<PathBuf, Deployment>::new()));

    let Some(main_package_fs_path) = package.package_path()?.fs_path else {
        anyhow::bail!("Can only deploy a local package");
    };

    let all_package_paths = {
        let mut all_packages = vec![main_package_fs_path];
        all_packages.extend(extra_packages.iter().map(|p| {
            PackagePath::new_local(p)
                .expect("Failed to construct local package path")
                .fs_path
                .expect("Failed to get FS path for local package")
        }));
        all_packages
    };

    let mut first_root_deployment = None;
    for package_path in all_package_paths {
        let skip_building = manifest_path_to_version.lock().keys().cloned().collect();

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
                //
                // Additionally, update the package's version if required.
                let manifest_path_to_version = manifest_path_to_version.clone();
                async move {
                    let package_path = package_manifest_path.parent().unwrap();

                    let mut manifest: toml_edit::Document =
                        tokio::fs::read_to_string(&package_manifest_path)
                            .await?
                            .parse()?;

                    let mut edited = false;

                    if let Some(dependencies) = manifest.as_table_mut().get_mut("dependencies") {
                        for (_, dependency) in dependencies.as_table_like_mut().unwrap().iter_mut()
                        {
                            let Some(dependency) = dependency.as_table_like_mut() else {
                                continue;
                            };
                            let Some(dependency_path) =
                                dependency.get("path").and_then(|i| i.as_str())
                            else {
                                continue;
                            };

                            let dependency_manifest_path = ambient_std::path::normalize(
                                &package_path.join(dependency_path).join("ambient.toml"),
                            );

                            if let Some((_, version)) = manifest_path_to_version
                                .lock()
                                .get(&dependency_manifest_path)
                                .and_then(|d| d.as_deployed())
                            {
                                dependency.insert("version", toml_edit::value(version));
                            }
                        }

                        edited = true;
                    }

                    if let Some(version) = &version {
                        manifest["package"]["version"] = toml_edit::value(version.clone());
                        edited = true;
                    }

                    if edited {
                        tokio::fs::write(&package_manifest_path, manifest.to_string()).await?;
                    }

                    Ok(())
                }
            },
            |package_manifest_path, build_path, was_built| {
                // After build, deploy the package.
                let manifest_path_to_version = manifest_path_to_version.clone();
                let package_path = package_manifest_path.parent().unwrap().to_owned();
                async move {
                    let deployment = if was_built {
                        let (_deployment_id, manifest) = ambient_deploy::deploy(
                            api_server,
                            token,
                            &build_path,
                            &package_path,
                            *force_upload,
                        )
                        .await?;
                        Deployment::Deployed {
                            package_id: manifest.package.id.expect("no package ID").to_string(),
                            version: manifest.package.version.to_string(),
                        }
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

                    manifest_path_to_version
                        .lock()
                        .insert(package_manifest_path.to_owned(), deployment);

                    Ok(())
                }
            },
        )
        .await?;

        let main_package_name = result.main_package_name;

        let deployment = manifest_path_to_version
            .lock()
            .get(&package_path.join("ambient.toml"))
            .cloned()
            .context("Main package was not processed; this is a bug")?;

        match &deployment {
            Deployment::Skipped => {
                tracing::info!(
                    "Package \"{main_package_name}\" was already deployed, skipping deployment"
                );
            }
            Deployment::Deployed {
                package_id,
                version,
            } => {
                let ensure_running_url = ambient_shared_types::urls::ensure_running_url(
                    ambient_shared_types::urls::ServerSelector::Package {
                        id: package_id,
                        version: Some(version.as_str()),
                    },
                );
                let web_url =
                    ambient_shared_types::urls::web_package_url(package_id, Some(version.as_str()));

                tracing::info!("Package \"{main_package_name}\" deployed successfully!");
                tracing::info!("  Join: ambient join '{ensure_running_url}'");
                tracing::info!("  Web URL: '{}'", web_url.bright_green());

                if first_root_deployment.is_none() {
                    first_root_deployment = Some(deployment);
                }
            }
        }
    }

    if let Some((package_id, version)) = first_root_deployment
        .filter(|_| *ensure_running)
        .and_then(|d| d.as_deployed())
    {
        let spec = ambient_cloud_client::ServerSpec::new_with_package(package_id, version)
            .with_context(context.to_string());
        let server =
            ambient_cloud_client::ensure_server_running(assets, api_server, token.into(), spec)
                .await?;
        tracing::info!("Deployed package is running at {}", server.host);
    }

    Ok(())
}
