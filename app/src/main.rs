use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ContentBaseUrlKey, UsingLocalDebugAssetsKey},
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
use cli::{build::BuildDirectories, Cli, Commands, PackagePath};
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

    let package = cli.package();

    if let Some(package) = package {
        if package.project {
            log::warn!("`-p`/`--project` has no semantic meaning.");
            log::warn!("You do not need to use `-p`/`--project` - `ambient run project` is the same as `ambient run -p project`.");
        }
    }

    let package_path: PackagePath = package.and_then(|p| p.path.clone()).try_into()?;
    let golden_image_output_dir = package_path.fs_path.clone();

    if package_path.is_remote() {
        // package path is a URL, so let's use it as the content base URL
        ContentBaseUrlKey.insert(&assets, package_path.url.clone());
    }

    // If new: create package, immediately exit
    if let Commands::New { name, api_path, .. } = &cli.command {
        return cli::new_package::handle(&package_path, name.as_deref(), api_path.as_deref())
            .context("Failed to create package");
    }

    if let Commands::Assets { command } = &cli.command {
        return rt.block_on(cli::assets::handle(command, &assets));
    }

    // Store a flag that we are using local debug assets
    // Used for emitting warnings when local debug assets are sent to remote clients
    UsingLocalDebugAssetsKey.insert(
        &assets,
        !package_path.is_remote() && !cli.use_release_build(),
    );

    // Build the package if required. Note that this only runs if the package is local,
    // and if a build has actually been requested.
    let BuildDirectories {
        build_root_path,
        main_package_path,
    } = rt.block_on(cli::build::build(
        package,
        package_path,
        &assets,
        cli.use_release_build(),
    ))?;

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
        if !cli.use_release_build() {
            log::warn!("Deploying a debug build which might involve uploading large files. Remove `--debug` to deploy a release build.");
        }
        return rt.block_on(cli::deploy::handle(
            &main_package_path,
            &assets,
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
            build_root_path,
            main_package_path,
            &assets,
        ))?
    } else {
        unreachable!()
    };

    // Time to join!
    if let Some(run) = cli.run() {
        cli::client::handle(run, &rt, assets, server_addr, golden_image_output_dir)?;
    } else {
        // Otherwise, wait for the Ctrl+C signal
        match rt.block_on(tokio::signal::ctrl_c()) {
            Ok(()) => {}
            Err(err) => log::error!("Unable to listen for shutdown signal: {}", err),
        }
    }

    Ok(())
}

// Read the package manifest from the package path (which may have been updated by the build step)
async fn retrieve_manifest(
    built_package_path: &AbsAssetUrl,
    assets: &AssetCache,
) -> anyhow::Result<ambient_package::Manifest> {
    match built_package_path
        .push("ambient.toml")?
        .download_string(assets)
        .await
    {
        Ok(toml) => Ok(ambient_package::Manifest::parse(&toml)?),
        Err(_) => {
            anyhow::bail!("Failed to find ambient.toml in package");
        }
    }
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
