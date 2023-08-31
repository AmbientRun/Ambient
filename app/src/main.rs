use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::{ContentBaseUrlKey, UsingLocalDebugAssetsKey},
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
use cli::{Cli, Commands};
use log::LevelFilter;
use serde::Deserialize;
use server::{ServerHandle, QUIC_INTERFACE_PORT};
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let rt = ambient_sys::task::make_native_multithreaded_runtime()?;

    setup_logging()?;

    shared::components::init()?;

    let runtime = rt.handle();
    let assets = AssetCache::new(runtime.clone());
    PhysicsKey.get(&assets); // Load physics
    AssetsCacheOnDisk.insert(&assets, false); // Disable disk caching for now; see https://github.com/AmbientRun/Ambient/issues/81

    let cli = if let Some(launch_json) = LaunchJson::load()? {
        Cli::parse_from(launch_json.args())
    } else {
        Cli::parse()
    };

    if let Some(package) = cli.package() {
        if package.project {
            log::warn!("`-p`/`--project` has no semantic meaning.");
            log::warn!("You do not need to use `-p`/`--project` - `ambient run project` is the same as `ambient run -p project`.");
        }
    }

    // Update some ~~global variables~~ asset keys with package-path derived state
    if let Some(package_path) = cli.package().map(|p| p.package_path()).transpose()? {
        if package_path.is_remote() {
            // package path is a URL, so let's use it as the content base URL
            ContentBaseUrlKey.insert(&assets, package_path.url.clone());
        }

        // Store a flag that we are using local debug assets
        // Used for emitting warnings when local debug assets are sent to remote clients
        UsingLocalDebugAssetsKey.insert(
            &assets,
            !package_path.is_remote() && !cli.use_release_build(),
        );
    }

    let release_build = cli.use_release_build();
    match &cli.command {
        // commands that immediately exit
        Commands::New {
            package,
            name,
            api_path,
        } => {
            let package_path = package.package_path()?;
            cli::new_package::handle(&package_path, name.as_deref(), api_path.as_deref())
                .context("Failed to create package")
        }
        Commands::Assets { command } => rt.block_on(cli::assets::handle(command, &assets)),
        Commands::Build { package } => rt.block_on(async {
            cli::build::handle(package, &assets, release_build)
                .await
                .map(|_| ())
        }),
        Commands::Deploy {
            package,
            extra_packages,
            api_server,
            token,
            force_upload,
            ensure_running,
            context,
        } => rt.block_on(async {
            cli::deploy::handle(
                package,
                extra_packages,
                &assets,
                token.as_deref(),
                api_server,
                release_build,
                *force_upload,
                *ensure_running,
                context,
            )
            .await
        }),
        Commands::Login => rt.block_on(cli::login::handle(&assets)),

        // client
        Commands::Join { run, host } => {
            let server_addr = rt.block_on(determine_join_addr(&assets, host.as_ref()))?;
            cli::client::handle(run, &rt, assets, server_addr, None)
        }

        // server
        Commands::Serve { package, host } => rt.block_on(async {
            let server_handle = run_server(&assets, release_build, package, host, None).await?;
            Ok(server_handle.join().await?)
        }),

        // client+server
        Commands::Run { package, host, run } => {
            run_client_and_server(&rt, assets, release_build, package, host, run, None)
        }
        Commands::View {
            package,
            host,
            run,
            asset_path,
        } => {
            let path = Some(asset_path.clone());
            run_client_and_server(&rt, assets, release_build, package, host, run, path)
        }
    }
}

async fn determine_join_addr(
    assets: &AssetCache,
    host: Option<&String>,
) -> anyhow::Result<ResolvedAddr> {
    match host.cloned() {
        Some(mut host) => {
            if host.starts_with("http://") || host.starts_with("https://") {
                tracing::info!("NOTE: Joining server by http url is still experimental and can be removed without warning.");

                let reqwest = &ReqwestClientKey.get(assets);
                host = reqwest.get(host).send().await?.text().await?;

                if host.is_empty() {
                    anyhow::bail!("Failed to resolve host");
                }
            }
            if !host.contains(':') {
                host = format!("{host}:{QUIC_INTERFACE_PORT}");
            }
            ResolvedAddr::lookup_host(&host).await
        }
        None => Ok(ResolvedAddr::localhost_with_port(QUIC_INTERFACE_PORT)),
    }
}

async fn run_server(
    assets: &AssetCache,
    release_build: bool,
    package: &cli::PackageCli,
    host: &cli::HostCli,
    view_asset_path: Option<PathBuf>,
) -> anyhow::Result<ServerHandle> {
    let dirs = cli::build::handle(package, assets, release_build).await?;
    cli::server::handle(host, view_asset_path, dirs, assets).await
}

fn run_client_and_server(
    rt: &tokio::runtime::Runtime,
    assets: AssetCache,
    release_build: bool,
    package: &cli::PackageCli,
    host: &cli::HostCli,
    run: &cli::RunCli,
    view_asset_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    let server_handle = rt.block_on(run_server(
        &assets,
        release_build,
        package,
        host,
        view_asset_path,
    ))?;

    let package_path = package.package_path()?;
    cli::client::handle(
        run,
        rt,
        assets,
        server_handle.resolve_as_localhost(),
        package_path.fs_path,
    )
}

#[derive(Deserialize)]
struct LaunchJson {
    args: Vec<String>,
}
impl LaunchJson {
    fn load() -> anyhow::Result<Option<Self>> {
        if std::env::args().len() > 1 {
            return Ok(None);
        }
        let mut launch_file = Path::new("launch.json").to_path_buf();
        if !launch_file.exists() {
            launch_file = std::env::current_dir()?.join("launch.json");
        }
        if !launch_file.exists() {
            if let Some(parent) = std::env::current_exe()?.parent() {
                launch_file = parent.join("launch.json");
            }
        }
        if !launch_file.exists() {
            return Ok(None);
        }
        log::info!("Using launch.json for CLI args: {}", launch_file.display());
        let launch_json =
            std::fs::read_to_string(launch_file).context("Failed to read launch.json")?;
        let launch_json: Self =
            serde_json::from_str(&launch_json).context("Failed to parse launch.json")?;
        Ok(Some(launch_json))
    }
    fn args(&self) -> Vec<String> {
        let mut args = std::env::args().collect::<Vec<_>>();
        [args.pop().unwrap()]
            .into_iter()
            .chain(self.args.iter().cloned())
            .collect()
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
            .with(
                tracing_subscriber::fmt::Layer::new().with_timer(
                    tracing_subscriber::fmt::time::LocalTime::new(
                        time::format_description::parse("[hour]:[minute]:[second]")
                            .expect("format string should be valid!"),
                    ),
                ),
            )
            .try_init()?;

        Ok(())
    }
}
