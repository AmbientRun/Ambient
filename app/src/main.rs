use std::{path::PathBuf, str::FromStr};

use ambient_core::window::ExitStatus;
use ambient_network::native::client::ResolvedAddr;
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ContentBaseUrlKey},
    download_asset::AssetsCacheOnDisk,
};
use clap::Parser;

mod cli;
mod client;
mod server;
mod shared;

use ambient_physics::physx::PhysicsKey;
use anyhow::{bail, Context};
use cli::{AssetCommand, Cli, Commands};
use log::LevelFilter;
use server::QUIC_INTERFACE_PORT;

#[cfg(not(feature = "no_bundled_certs"))]
const CERT: &[u8] = include_bytes!("../../localhost.crt");

#[cfg(not(feature = "no_bundled_certs"))]
const CERT_KEY: &[u8] = include_bytes!("../../localhost.key");

fn setup_logging() -> anyhow::Result<()> {
    const MODULES: &[(LevelFilter, &[&str])] = &[
        (
            LevelFilter::Error,
            &[
                // Warns about extra syntactic elements; we are not concerned with these.
                "fbxcel",
            ],
        ),
        (
            LevelFilter::Warn,
            &[
                "ambient_build",
                "ambient_gpu",
                "ambient_model",
                "ambient_physics",
                "ambient_std",
                "cranelift_codegen",
                "naga",
                "tracing",
                "symphonia_core",
                "symphonia_bundle_mp3",
                "wgpu_core",
                "wgpu_hal",
            ],
        ),
    ];

    // Initialize the logger and lower the log level for modules we don't need to hear from by default.
    #[cfg(not(feature = "tracing"))]
    {
        let mut builder = env_logger::builder();
        builder.filter_level(LevelFilter::Info);

        for (level, modules) in MODULES {
            for module in *modules {
                builder.filter_module(module, *level);
            }
        }

        builder.parse_default_env().try_init()?;

        Ok(())
    }

    #[cfg(feature = "tracing")]
    {
        use tracing::metadata::Level;
        use tracing_log::AsTrace;
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::{registry, EnvFilter};

        let mut filter = tracing_subscriber::filter::Targets::new()
            .with_default(tracing::metadata::LevelFilter::DEBUG);
        for (level, modules) in MODULES {
            for &module in *modules {
                filter = filter.with_target(module, level.as_trace());
            }
        }

        // BLOCKING: pending https://github.com/tokio-rs/tracing/issues/2507
        // let modules: Vec<_> = MODULES.iter().flat_map(|&(level, modules)| modules.iter().map(move |&v| format!("{v}={level}"))).collect();

        // eprintln!("{modules:#?}");
        // let mut filter = tracing_subscriber::filter::EnvFilter::builder().with_default_directive(Level::INFO.into()).from_env_lossy();

        // for module in modules {
        //     filter = filter.add_directive(module.parse().unwrap());
        // }

        // let mut filter = std::env::var("RUST_LOG").unwrap_or_default().parse::<tracing_subscriber::filter::Targets>().unwrap_or_default();
        // filter.extend(MODULES.iter().flat_map(|&(level, modules)| modules.iter().map(move |&v| (v, level.as_trace()))));

        let env_filter = EnvFilter::builder()
            .with_default_directive(Level::INFO.into())
            .from_env_lossy();

        let layered_registry = registry().with(filter).with(env_filter);

        // use stackdriver format if available and requested
        #[cfg(feature = "stackdriver")]
        if std::env::var("LOG_FORMAT").unwrap_or_default() == "stackdriver" {
            layered_registry
                .with(tracing_stackdriver::layer().with_writer(std::io::stdout))
                .try_init()?;
            return Ok(());
        }

        // otherwise use the default format
        layered_registry
            .with(tracing_subscriber::fmt::Layer::new().with_timer(
                tracing_subscriber::fmt::time::LocalTime::new(time::macros::format_description!(
                    "[hour]:[minute]:[second]"
                )),
            ))
            .try_init()?;

        Ok(())
    }
}

struct ProjectPath {
    url: AbsAssetUrl,
    fs_path: Option<std::path::PathBuf>,
}

impl ProjectPath {
    fn new_local(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let current_dir = std::env::current_dir().context("Error getting current directory")?;
        let path = if path.is_absolute() {
            path
        } else {
            ambient_std::path::normalize(&current_dir.join(path))
        };

        if path.exists() && !path.is_dir() {
            anyhow::bail!("Project path {path:?} exists and is not a directory.");
        }
        let url = AbsAssetUrl::from_directory_path(path);
        let fs_path = url.to_file_path().ok().flatten();

        Ok(Self { url, fs_path })
    }

    fn is_local(&self) -> bool {
        self.fs_path.is_some()
    }

    fn is_remote(&self) -> bool {
        self.fs_path.is_none()
    }

    // 'static to limit only to compile-time known paths
    fn push(&self, path: &'static str) -> AbsAssetUrl {
        self.url.push(path).unwrap()
    }
}

impl TryFrom<Option<String>> for ProjectPath {
    type Error = anyhow::Error;

    fn try_from(project_path: Option<String>) -> anyhow::Result<Self> {
        match project_path {
            Some(project_path)
                if project_path.starts_with("http://") || project_path.starts_with("https://") =>
            {
                let url = AbsAssetUrl::from_str(&project_path)?;
                Ok(Self { url, fs_path: None })
            }
            Some(project_path) => Self::new_local(project_path),
            None => {
                let url = AbsAssetUrl::from_directory_path(std::env::current_dir()?);
                let fs_path = url.to_file_path().ok().flatten();
                Ok(Self { url, fs_path })
            }
        }
    }
}

async fn load_manifest(
    assets: &AssetCache,
    path: &ProjectPath,
) -> anyhow::Result<ambient_project::Manifest> {
    if let Some(path) = &path.fs_path {
        // load manifest from file
        Ok(
            ambient_project::Manifest::from_file(path.join("ambient.toml"))
                .context("Failed to read ambient.toml.")?,
        )
    } else {
        // path is a URL, so download the pre-build manifest (with resolved imports)
        let manifest_url = path.url.push("build/ambient.toml").unwrap();
        let manifest_data = manifest_url
            .download_string(assets)
            .await
            .context("Failed to download ambient.toml.")?;

        let manifest = ambient_project::Manifest::parse(&manifest_data)
            .context("Failed to parse downloaded ambient.toml.")?;
        Ok(manifest)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging()?;

    shared::components::init()?;
    // let runtime = tokio::runtime::Builder::new_multi_thread()
    //     .enable_all()
    //     .build()?;
    let runtime = tokio::runtime::Handle::current();
    let assets = AssetCache::new(runtime.clone());
    PhysicsKey.get(&assets); // Load physics
    AssetsCacheOnDisk.insert(&assets, false); // Disable disk caching for now; see https://github.com/AmbientRun/Ambient/issues/81

    let cli = Cli::parse();

    let project = cli.project();

    let project_path: ProjectPath = project.and_then(|p| p.path.clone()).try_into()?;
    if project_path.is_remote() {
        // project path is a URL, so let's use it as the content base URL
        ContentBaseUrlKey.insert(&assets, project_path.url.push("build/")?);
    }

    // If new: create project, immediately exit
    if let Commands::New { name, api_path, .. } = &cli.command {
        if let Some(path) = &project_path.fs_path {
            if let Err(err) =
                cli::new_project::new_project(path, name.as_deref(), api_path.as_deref())
            {
                eprintln!("Failed to create project: {err:?}");
            }
        } else {
            eprintln!("Cannot create project in a remote directory.");
        }
        return Ok(());
    }

    if let Commands::Assets { command, path } = &cli.command {
        let path = ProjectPath::new_local(path.clone())?;
        let manifest = load_manifest(&assets, &path).await?;

        match command {
            AssetCommand::MigratePipelinesToml => {
                ambient_build::migrate::toml::process(&manifest, path.fs_path.unwrap())
                    .await
                    .context("Failed to migrate pipelines")?;
            }
        }

        return Ok(());
    }

    // If a project was specified, assume that assets need to be built
    let manifest = match project {
        Some(_) => Some(load_manifest(&assets, &project_path).await?),
        None => None,
    };

    let metadata = if let Some(manifest) = manifest.as_ref() {
        let project = project.unwrap();

        if !project.no_build && project_path.is_local() {
            let project_name = manifest.ember.name.as_deref().unwrap_or("project");

            tracing::info!("Building project {:?}", project_name);

            let metadata = ambient_build::build(
                PhysicsKey.get(&assets),
                &assets,
                project_path
                    .fs_path
                    .clone()
                    .expect("should be present as it's already checked above"),
                manifest,
                project.release,
                project.clean_build,
            )
            .await
            .context("Failed to build project")?;

            Some(metadata)
        } else {
            let metadata_url = project_path.push("build/metadata.toml");
            let metadata_data = metadata_url
                .download_string(&assets)
                .await
                .context("Failed to load build/metadata.toml.")?;

            Some(ambient_build::Metadata::parse(&metadata_data)?)
        }
    } else {
        None
    };

    // If this is just a build, exit now
    if matches!(&cli.command, Commands::Build { .. }) {
        return Ok(());
    }

    // If this is just a deploy then deploy and exit
    #[cfg(feature = "deploy")]
    if let Commands::Deploy {
        token,
        api_server,
        force_upload,
        ..
    } = &cli.command
    {
        let Some(auth_token) = token else {
            anyhow::bail!("-t/--token is required for deploy");
        };
        let Some(project_fs_path) = &project_path.fs_path else {
            anyhow::bail!("Can only deploy a local project");
        };
        let manifest = manifest.as_ref().expect("no manifest");
        let deployment_id = ambient_deploy::deploy(
            &runtime,
            api_server
                .clone()
                .unwrap_or("https://api.ambient.run".to_string()),
            auth_token,
            project_fs_path,
            manifest,
            *force_upload,
        )
        .await?;
        log::info!(
            "Assets deployed successfully. Deployment id: {}. Deploy url: https://assets.ambient.run/{}",
            deployment_id,
            deployment_id,
        );
        return Ok(());
    }

    // Otherwise, either connect to a server or host one
    let server_addr = if let Commands::Join { host, .. } = &cli.command {
        if let Some(mut host) = host.clone() {
            if !host.contains(':') {
                host = format!("{host}:{QUIC_INTERFACE_PORT}");
            }
            ResolvedAddr::lookup_host(&host).await?
        } else {
            ResolvedAddr::localhost_with_port(QUIC_INTERFACE_PORT)
        }
    } else if let Some(host) = &cli.host() {
        let crypto = if let (Some(cert_file), Some(key_file)) = (&host.cert, &host.key) {
            let raw_cert = std::fs::read(cert_file).context("Failed to read certificate file")?;
            let cert_chain = if raw_cert.starts_with(b"-----BEGIN CERTIFICATE-----") {
                rustls_pemfile::certs(&mut raw_cert.as_slice())
                    .context("Failed to parse certificate file")?
            } else {
                vec![raw_cert]
            };
            let raw_key = std::fs::read(key_file).context("Failed to read certificate key")?;
            let key = if raw_key.starts_with(b"-----BEGIN ") {
                rustls_pemfile::read_all(&mut raw_key.as_slice())
                    .context("Failed to parse certificate key")?
                    .into_iter()
                    .find_map(|item| match item {
                        rustls_pemfile::Item::RSAKey(key) => Some(key),
                        rustls_pemfile::Item::PKCS8Key(key) => Some(key),
                        rustls_pemfile::Item::ECKey(key) => Some(key),
                        _ => None,
                    })
                    .ok_or_else(|| anyhow::anyhow!("No private key found"))?
            } else {
                raw_key
            };
            ambient_network::native::server::Crypto { cert_chain, key }
        } else {
            #[cfg(feature = "no_bundled_certs")]
            {
                anyhow::bail!("--cert and --key are required without bundled certs.");
            }
            #[cfg(not(feature = "no_bundled_certs"))]
            {
                tracing::info!("Using bundled certificate and key");
                ambient_network::native::server::Crypto {
                    cert_chain: vec![CERT.to_vec()],
                    key: CERT_KEY.to_vec(),
                }
            }
        };

        let addr = server::start(
            &runtime,
            assets.clone(),
            cli.clone(),
            project_path.url,
            manifest.as_ref().expect("no manifest"),
            metadata.as_ref().expect("no build metadata"),
            crypto,
        )
        .await;

        ResolvedAddr::localhost_with_port(addr.port())
    } else {
        unreachable!()
    };

    // Time to join!

    if let Some(run) = cli.run() {
        // If we have run parameters, start a client and join a server
        let exit_status = client::run(assets, server_addr, run, project_path.fs_path).await;
        if exit_status == ExitStatus::FAILURE {
            bail!("client::run failed with {exit_status:?}");
        }
    } else {
        // Otherwise, wait for the Ctrl+C signal
        match tokio::signal::ctrl_c().await {
            Ok(()) => {}
            Err(err) => log::error!("Unable to listen for shutdown signal: {}", err),
        }
    }

    Ok(())
}
