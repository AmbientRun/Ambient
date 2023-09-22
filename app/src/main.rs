use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::{ContentBaseUrlKey, UsingLocalDebugAssetsKey},
    download_asset::AssetsCacheOnDisk,
};
use ambient_settings::SettingsKey;
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
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let rt = ambient_sys::task::make_native_multithreaded_runtime()?;

    setup_logging()?;

    shared::components::init()?;

    let runtime = rt.handle();
    let assets = AssetCache::new(runtime.clone());
    let _settings = SettingsKey.get(&assets);

    // _guard and _handle need to be kept around for the lifetime of the application
    let _guard: sentry::ClientInitGuard;
    let _handle: Result<sentry_rust_minidump::ClientHandle, sentry_rust_minidump::Error>;
    #[cfg(feature = "production")]
    if _settings.general.sentry.enabled {
        let sentry_dsn = _settings.general.sentry.dsn;
        _guard = init_sentry(&sentry_dsn);
        _handle = sentry_rust_minidump::init(&_guard);
        match _handle {
            Ok(_) => log::debug!("Initialized Sentry with DSN: {:?}", sentry_dsn),
            Err(err) => log::warn!("Failed to initialize Sentry: {:?}", err),
        }
    }

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
    let use_release_build = cli.use_release_build();
    if let Some(package_path) = cli.package().map(|p| p.package_path()).transpose()? {
        if package_path.is_remote() {
            // package path is a URL, so let's use it as the content base URL
            ContentBaseUrlKey.insert(&assets, package_path.url.clone());
        }

        // Store a flag that we are using local debug assets
        // Used for emitting warnings when local debug assets are sent to remote clients
        UsingLocalDebugAssetsKey.insert(&assets, !package_path.is_remote() && !use_release_build);
    }

    match &cli.command {
        // package commands
        Commands::Package { package } => cli::package::handle(package, &rt, assets),
        Commands::New(args) => cli::package::new::handle(args).context("Failed to create package"),
        Commands::Build(build) => rt.block_on(async {
            cli::package::build::handle(build, &assets, use_release_build)
                .await
                .map(|_| ())
        }),
        Commands::Deploy(deploy) => rt.block_on(cli::package::deploy::handle(
            deploy,
            &assets,
            use_release_build,
        )),
        Commands::Serve(serve) => rt.block_on(async {
            Ok(
                cli::package::serve::handle(serve, assets, use_release_build)
                    .await?
                    .join()
                    .await?,
            )
        }),
        Commands::Run(run) => cli::package::run::handle(&rt, run, assets, use_release_build),

        // non-package commands
        Commands::Assets { assets: command } => rt.block_on(cli::assets::handle(command, &assets)),
        Commands::Login => rt.block_on(cli::login::handle(&assets)),
        Commands::Join(join) => cli::join::handle(join, &rt, assets),
    }
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

#[cfg(feature = "production")]
fn init_sentry(sentry_dsn: &String) -> sentry::ClientInitGuard {
    std::env::set_var("RUST_BACKTRACE", "1"); // This is needed for anyhow errors captured by sentry to get backtraces

    // https://stackoverflow.com/questions/66790155/what-is-the-recommended-way-to-propagate-panics-in-rust-tokio-code
    // The "sentry" panic handler will call this
    // This is instead of using panic=abort, which doesn't work with workspace projects: https://github.com/rust-lang/cargo/issues/8264
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        std::process::exit(1);
    }));

    let version = ambient_native_std::ambient_version();
    sentry::init((
        sentry_dsn.to_owned(),
        sentry::ClientOptions {
            release: Some(format!("{}_{}", version.version, version.revision).into()),
            ..Default::default()
        },
    ))
}
