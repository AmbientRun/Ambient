use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use ambient_app::AppBuilder;
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_debugger::Debugger;
use ambient_ecs::{EntityData, SystemGroup};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::{
    client::{GameClient, GameClientNetworkStats, GameClientRenderTarget, GameClientServerStats, GameClientView, UseOnce},
    events::ServerEventRegistry,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    cb,
    download_asset::AssetsCacheOnDisk,
    friendly_id,
};
use ambient_ui::{use_window_physical_resolution, Dock, FocusRoot, StylesExt, Text, WindowSized};
use clap::{Args, Parser, Subcommand};

pub mod components;
mod new_project;
mod player;
mod server;

use ambient_physics::physx::PhysicsKey;
use anyhow::Context;
use log::LevelFilter;
use player::PlayerRawInputHandler;
use server::QUIC_INTERFACE_PORT;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Create a new Ambient project
    New {
        #[command(flatten)]
        project_args: ProjectCli,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Builds and runs the project locally
    Run {
        #[command(flatten)]
        project_args: ProjectCli,
        #[command(flatten)]
        host_args: HostCli,
        #[command(flatten)]
        run_args: RunCli,
    },
    /// Builds the project
    Build {
        #[command(flatten)]
        project_args: ProjectCli,
    },
    /// Builds and runs the project in server-only mode
    Serve {
        #[command(flatten)]
        project_args: ProjectCli,
        #[command(flatten)]
        host_args: HostCli,
    },
    /// View an asset
    View {
        #[command(flatten)]
        project_args: ProjectCli,
        /// Relative to the project path
        asset_path: PathBuf,
    },
    /// Join a multiplayer session
    Join {
        #[command(flatten)]
        run_args: RunCli,
        /// The server to connect to; defaults to localhost
        host: Option<String>,
    },
    /// Updates all WASM APIs with the core primitive components (not for users)
    #[cfg(not(feature = "production"))]
    #[command(hide = true)]
    UpdateInterfaceComponents,
}
#[derive(Args, Clone)]
struct RunCli {
    /// Whether or not debug menus should be shown
    #[arg(long)]
    debug: bool,

    /// The user ID to join this server with
    #[clap(short, long)]
    user_id: Option<String>,
}
#[derive(Args, Clone)]
struct ProjectCli {
    /// The path of the project to run; if not specified, this will default to the current directory
    path: Option<PathBuf>,
}
#[derive(Args, Clone)]
struct HostCli {
    /// Provide a public address or IP to the instance, which will allow users to connect to this instance over the internet
    ///
    /// Defaults to localhost
    #[arg(long)]
    public_host: Option<String>,
}

impl Commands {
    /// Extract run-relevant state only
    fn run(&self) -> Option<&RunCli> {
        match self {
            Commands::New { .. } => None,
            Commands::Run { run_args, .. } => Some(run_args),
            Commands::Build { .. } => None,
            Commands::Serve { .. } => None,
            Commands::View { .. } => None,
            Commands::Join { run_args, .. } => Some(run_args),
            #[cfg(not(feature = "production"))]
            Commands::UpdateInterfaceComponents => None,
        }
    }
    /// Extract project-relevant state only
    fn project(&self) -> Option<&ProjectCli> {
        match self {
            Commands::New { project_args, .. } => Some(project_args),
            Commands::Run { project_args, .. } => Some(project_args),
            Commands::Build { project_args, .. } => Some(project_args),
            Commands::Serve { project_args, .. } => Some(project_args),
            Commands::View { project_args, .. } => Some(project_args),
            Commands::Join { .. } => None,
            #[cfg(not(feature = "production"))]
            Commands::UpdateInterfaceComponents => None,
        }
    }
    /// Extract host-relevant state only
    fn host(&self) -> Option<&HostCli> {
        match self {
            Commands::New { .. } => None,
            Commands::Run { host_args, .. } => Some(host_args),
            Commands::Build { .. } => None,
            Commands::Serve { host_args, .. } => Some(host_args),
            Commands::View { .. } => None,
            Commands::Join { .. } => None,
            #[cfg(not(feature = "production"))]
            Commands::UpdateInterfaceComponents => None,
        }
    }
}

fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![
            Box::new(ambient_decals::client_systems()),
            Box::new(ambient_primitives::systems()),
            Box::new(ambient_sky::systems()),
            Box::new(ambient_water::systems()),
            Box::new(ambient_physics::client_systems()),
            Box::new(player::client_systems()),
        ],
    )
}

#[element_component]
fn GameView(hooks: &mut Hooks, show_debug: bool) -> Element {
    let (state, _) = hooks.consume_context::<GameClient>().unwrap();
    let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();

    if show_debug {
        Debugger {
            get_state: cb(move |cb| {
                let mut game_state = state.game_state.lock();
                let game_state = &mut *game_state;
                cb(&mut game_state.renderer, &render_target.0, &mut game_state.world);
            }),
        }
        .el()
    } else {
        Element::new()
    }
}

#[element_component]
fn MainApp(hooks: &mut Hooks, server_addr: SocketAddr, user_id: String, show_debug: bool) -> Element {
    let resolution = use_window_physical_resolution(hooks);

    hooks.provide_context(GameClientNetworkStats::default);
    hooks.provide_context(GameClientServerStats::default);

    FocusRoot::el([
        UICamera.el().set(active_camera(), 0.),
        PlayerRawInputHandler.el(),
        WindowSized::el([GameClientView {
            server_addr,
            user_id,
            resolution,
            on_disconnect: cb(move || {}),
            init_world: cb(UseOnce::new(Box::new(move |world, _render_target| {
                world.add_resource(ambient_network::events::event_registry(), Arc::new(ServerEventRegistry::new()));
            }))),
            on_loaded: cb(move |_game_state, _game_client| Ok(Box::new(|| {}))),
            error_view: cb(move |error| Dock(vec![Text::el("Error").header_style(), Text::el(error)]).el()),
            systems_and_resources: cb(|| (client_systems(), EntityData::new())),
            create_rpc_registry: cb(server::create_rpc_registry),
            on_in_entities: None,
            ui: GameView { show_debug }.el(),
        }
        .el()]),
    ])
}

fn main() -> anyhow::Result<()> {
    // Initialize the logger and lower the log level for modules we don't need to hear from by default.
    {
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
                    "ambient_network",
                    "ambient_physics",
                    "ambient_std",
                    "naga",
                    "tracing",
                    "wgpu_core",
                    "wgpu_hal",
                ],
            ),
        ];

        let mut builder = env_logger::builder();
        builder.filter_level(LevelFilter::Info);

        for (level, modules) in MODULES {
            for module in *modules {
                builder.filter_module(module, *level);
            }
        }

        builder.parse_default_env().try_init()?;
    }
    components::init()?;
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    let assets = AssetCache::new(runtime.handle().clone());
    PhysicsKey.get(&assets); // Load physics
    AssetsCacheOnDisk.insert(&assets, false); // Disable disk caching for now; see https://github.com/AmbientRun/Ambient/issues/81

    let cli = Cli::parse();

    let current_dir = std::env::current_dir()?;
    let project_path = cli.command.project().and_then(|p| p.path.clone()).unwrap_or_else(|| current_dir.clone());
    let project_path =
        if project_path.is_absolute() { project_path } else { ambient_std::path::normalize(&current_dir.join(project_path)) };

    if project_path.exists() && !project_path.is_dir() {
        anyhow::bail!("Project path {project_path:?} exists and is not a directory.");
    }

    // If new: create project, immediately exit
    if let Commands::New { name, .. } = cli.command {
        if let Err(err) = new_project::new_project(&project_path, name.as_deref()) {
            eprintln!("Failed to create project: {err:?}");
        }
        return Ok(());
    }

    // If UIC: write components to disk, immediately exit
    #[cfg(not(feature = "production"))]
    if let Commands::UpdateInterfaceComponents = cli.command {
        let toml = components::dev::build_components_toml().to_string();

        // Assume we are being run within the codebase.
        for guest_path in std::fs::read_dir("guest/").unwrap().filter_map(Result::ok).map(|de| de.path()).filter(|de| de.is_dir()) {
            let toml_path = if guest_path.file_name().unwrap_or_default() == "rust" {
                guest_path.join("api").join("api_macros").join("ambient.toml")
            } else {
                guest_path.join("api").join("ambient.toml")
            };
            std::fs::write(&toml_path, &toml)?;
            log::info!("Interface updated at {toml_path:?}");
        }
        return Ok(());
    }

    // If a project was specified, assume that assets need to be built
    let manifest = cli
        .command
        .project()
        .map(|_| {
            anyhow::Ok(ambient_project::Manifest::parse(
                &std::fs::read_to_string(project_path.join("ambient.toml")).context("No project manifest was found. Please create one.")?,
            )?)
        })
        .transpose()?;

    if let Some(manifest) = manifest.as_ref() {
        let project_name = manifest.project.name.as_deref().unwrap_or("project");
        log::info!("Building {}", project_name);
        runtime.block_on(ambient_build::build(PhysicsKey.get(&assets), &assets, project_path.clone(), manifest));
        log::info!("Done building {}", project_name);
    }

    // If this is just a build, exit now
    if matches!(&cli.command, Commands::Build { .. }) {
        return Ok(());
    }

    // Otherwise, either connect to a server or host one
    let server_addr = if let Commands::Join { host, .. } = &cli.command {
        if let Some(mut host) = host.clone() {
            if !host.contains(':') {
                host = format!("{host}:{QUIC_INTERFACE_PORT}");
            }
            host.parse().with_context(|| format!("Invalid address for host {host}"))?
        } else {
            format!("127.0.0.1:{QUIC_INTERFACE_PORT}").parse()?
        }
    } else {
        let port = server::start_server(&runtime, assets.clone(), cli.clone(), project_path, manifest.as_ref().expect("no manifest"));
        format!("127.0.0.1:{port}").parse()?
    };

    // Time to join!
    let handle = runtime.handle().clone();
    if let Some(run) = cli.command.run() {
        // If we have run parameters, start a client and join a server
        let user_id = run.user_id.clone().unwrap_or_else(|| format!("user_{}", friendly_id()));
        AppBuilder::simple().ui_renderer(true).with_runtime(runtime).with_asset_cache(assets).run(|app, _runtime| {
            MainApp { server_addr, user_id, show_debug: run.debug }.el().spawn_interactive(&mut app.world);
        });
    } else {
        // Otherwise, wait for the Ctrl+C signal
        handle.block_on(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {}
                Err(err) => log::error!("Unable to listen for shutdown signal: {}", err),
            }
        });
    }
    Ok(())
}
