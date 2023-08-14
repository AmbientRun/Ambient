use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ContentBaseUrlKey},
    download_asset::{AssetsCacheOnDisk, ReqwestClientKey},
};
use ambient_network::native::client::ResolvedAddr;
use clap::Parser;

mod cli;
mod client;
mod server;
mod shared;

use ambient_physics::physx::PhysicsKey;
use anyhow::Context;
use cli::{Cli, Commands, ProjectPath};
use log::LevelFilter;
use server::QUIC_INTERFACE_PORT;

fn main() -> anyhow::Result<()> {
    let rt = ambient_sys::task::make_native_multithreaded_runtime()?;

    setup_logging()?;

    shared::components::init()?;

    let runtime = rt.handle();
    let assets = AssetCache::new(runtime.clone());
    PhysicsKey.get(&assets); // Load physics
    AssetsCacheOnDisk.insert(&assets, false); // Disable disk caching for now; see https://github.com/AmbientRun/Ambient/issues/81

    let cli = Cli::parse();

    let project = cli.project();

    if let Some(project) = project {
        if project.project {
            log::warn!("`-p`/`--project` has no semantic meaning - the path is always treated as a project path.");
            log::warn!("You do not need to use `-p`/`--project` - `ambient run project` is the same as `ambient run -p project`.");
        }
    }

    let project_path: ProjectPath = project.and_then(|p| p.path.clone()).try_into()?;

    if project_path.is_remote() {
        // project path is a URL, so let's use it as the content base URL
        ContentBaseUrlKey.insert(&assets, project_path.url.push("build/")?);
    }

    // If new: create project, immediately exit
    if let Commands::New { name, api_path, .. } = &cli.command {
        return cli::new_project::handle(&project_path, name.as_deref(), api_path.as_deref())
            .context("Failed to create project");
    }

    if let Commands::Assets { command } = &cli.command {
        return rt.block_on(cli::assets::handle(command, &assets));
    }

    // Build the project if required. Note that this only runs if the project is local,
    // and if a build has actually been requested.
    //
    // Update the project path to match the build path if necessary.
    let original_project_path = project_path.clone();
    let (project_path, build_path) =
        rt.block_on(cli::build::build(project, project_path, &assets))?;

    // If this is just a build, exit now
    if matches!(&cli.command, Commands::Build { .. }) {
        return Ok(());
    }

    // If this is just a deploy then deploy and exit
    if let Commands::Deploy {
        token,
        api_server,
        force_upload,
        ensure_running,
        context,
        ..
    } = &cli.command
    {
        return rt.block_on(cli::deploy::handle(
            &project_path,
            &assets,
            build_path.as_ref(),
            token,
            api_server,
            *force_upload,
            *ensure_running,
            context,
        ));
    }

    // Otherwise, either connect to a server or host one
    let server_addr = if let Commands::Join { host, .. } = &cli.command {
        if let Some(mut host) = host.clone() {
            rt.block_on(async {
                if host.starts_with("http://") || host.starts_with("https://") {
                    tracing::info!("NOTE: Joining server by http url is still experimental and can be removed without warning.");

                    host = ReqwestClientKey
                        .get(&assets)
                        .get(host)
                        .send()
                        .await?
                        .text()
                        .await?;
                    if host.is_empty() {
                        anyhow::bail!("Failed to resolve host");
                    }
                }
                if !host.contains(':') {
                    host = format!("{host}:{QUIC_INTERFACE_PORT}");
                }
                ResolvedAddr::lookup_host(&host).await
            })?
        } else {
            ResolvedAddr::localhost_with_port(QUIC_INTERFACE_PORT)
        }
    } else if let Some(host) = &cli.host() {
        rt.block_on(cli::server::handle(
            host,
            if let cli::Commands::View { asset_path, .. } = &cli.command {
                Some(asset_path.clone())
            } else {
                None
            },
            project_path,
            build_path,
            &assets,
        ))?
    } else {
        unreachable!()
    };

    // Time to join!
    if let Some(run) = cli.run() {
        cli::client::handle(run, &rt, assets, server_addr, original_project_path)?;
    } else {
        // Otherwise, wait for the Ctrl+C signal
        match rt.block_on(tokio::signal::ctrl_c()) {
            Ok(()) => {}
            Err(err) => log::error!("Unable to listen for shutdown signal: {}", err),
        }
    }

    Ok(())
}

// Read the project manifest from the project path (which may have been updated by the build step)
// We attempt both the root and build/ as `ambient.toml` is in the former for local builds,
// and in the latter for deployed builds. This will likely be improved if/when deployments
// no longer have their own build directory.
async fn retrieve_project_path_and_manifest(
    project_path: &ProjectPath,
    assets: &AssetCache,
    build_path: Option<&AbsAssetUrl>,
) -> anyhow::Result<(ProjectPath, ambient_project::Manifest, AbsAssetUrl)> {
    async fn get_new_project_path_and_manifest(
        project_path: &ProjectPath,
        assets: &AssetCache,
    ) -> anyhow::Result<(ProjectPath, ambient_project::Manifest)> {
        let paths = [project_path.url.clone(), project_path.push("build")];

        for path in &paths {
            if let Ok(toml) = path.push("ambient.toml")?.download_string(assets).await {
                return Ok((
                    Some(path.to_string()).try_into()?,
                    ambient_project::Manifest::parse(&toml)?,
                ));
            }
        }

        anyhow::bail!("Failed to find ambient.toml in project");
    }

    let (project_path, manifest) = get_new_project_path_and_manifest(project_path, assets).await?;
    let build_path = build_path
        .cloned()
        .unwrap_or_else(|| project_path.url.push("build").unwrap());
    Ok((project_path, manifest, build_path))
}

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
                "ambient_gpu",
                "ambient_model",
                "ambient_physics",
                "ambient_native_std",
                "cranelift_codegen",
                "naga",
                "tracing",
                "symphonia_core",
                "symphonia_bundle_mp3",
                "wgpu_core",
                "wgpu_hal",
                "optivorbis",
                "symphonia_format_wav",
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
