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
use serde::Deserialize;
use std::path::Path;
use tracing_subscriber::{filter::LevelFilter, registry, EnvFilter};

fn main() -> anyhow::Result<()> {
    let rt = ambient_sys::task::make_native_multithreaded_runtime()?;

    setup_logging()?;

    ambient_git_rev_init::init().expect("Should be called exactly once");

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
            Ok(_) => tracing::debug!("Initialized Sentry with DSN: {:?}", sentry_dsn),
            Err(err) => tracing::warn!("Failed to initialize Sentry: {:?}", err),
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
            tracing::warn!("`-p`/`--project` has no semantic meaning.");
            tracing::warn!("You do not need to use `-p`/`--project` - `ambient run project` is the same as `ambient run -p project`.");
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
        Commands::New(args) => rt
            .block_on(cli::package::new::handle(args, &assets))
            .context("Failed to create package"),
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
        tracing::info!("Using launch.json for CLI args: {}", launch_file.display());
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
    // This fixes the `<unknown time>` in log formatting, an alternative is to use UTC time
    unsafe { time::util::local_offset::set_soundness(time::util::local_offset::Soundness::Unsound) }

    /// Create a layer of filtering before to remove info statements from external crates
    ///
    /// In general, `info` level logs should not be used in libraries as they are user facing and are recommended to be `debug` or `trace` events instead.
    const MODULES: &[(LevelFilter, &[&str])] = &[
        (
            LevelFilter::ERROR,
            &[
                // Warns about extra syntactic elements; we are not concerned with these.
                "fbxcel",
            ],
        ),
        (
            LevelFilter::WARN,
            &[
                "cranelift_codegen",
                "naga",
                "tracing",
                "symphonia_core",
                "symphonia_bundle_mp3",
                // TODO: remove, fixed in later wgpu version in https://github.com/gfx-rs/wgpu/commit/4478c52debcab1b88b80756b197dc10ece90dec9
                "wgpu_core",
                "wgpu_hal",
                "symphonia_format_wav",
            ],
        ),
    ];

    use tracing::metadata::Level;
    use tracing_subscriber::prelude::*;

    let mut targets = tracing_subscriber::filter::Targets::new()
        .with_default(tracing::metadata::LevelFilter::DEBUG);
    for (level, modules) in MODULES {
        for &module in *modules {
            targets = targets.with_target(module, *level);
        }
    }

    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    let registry = registry().with(targets).with(env_filter);

    // use stackdriver format if available and requested
    #[cfg(feature = "stackdriver")]
    if std::env::var("LOG_FORMAT").unwrap_or_default() == "stackdriver" {
        layered_registry
            .with(tracing_stackdriver::layer().with_writer(std::io::stdout))
            .try_init()?;
        return Ok(());
    }

    #[cfg(all(not(feature = "stackdriver"), not(feature = "tracing-tree")))]
    let format_layer = tracing_subscriber::fmt::Layer::new().compact().with_timer(
        tracing_subscriber::fmt::time::LocalTime::new(
            time::format_description::parse("[hour]:[minute]:[second]")
                .expect("format string should be valid!"),
        ),
    );

    #[cfg(feature = "tracing-tree")]
    let format_layer = tracing_tree::HierarchicalLayer::default()
        .with_targets(false)
        .with_indent_lines(true)
        .with_span_retrace(true)
        .with_deferred_spans(true);

    // otherwise use the default format
    registry.with(format_layer).try_init()?;

    Ok(())
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
