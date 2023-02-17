use std::{net::SocketAddr, sync::Arc};

use clap::{Parser, Subcommand};
use kiwi_app::AppBuilder;
use kiwi_cameras::UICamera;
use kiwi_core::camera::active_camera;
use kiwi_debugger::Debugger;
use kiwi_ecs::{EntityData, SystemGroup, World};
use kiwi_element::{element_component, Element, ElementComponentExt, Hooks};
use kiwi_network::{
    client::{GameClient, GameClientNetworkStats, GameClientRenderTarget, GameClientServerStats, GameClientView, UseOnce},
    events::ServerEventRegistry,
};
use kiwi_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    cb, friendly_id,
};
use kiwi_ui::{use_window_physical_resolution, Dock, FocusRoot, StylesExt, Text, WindowSized};

pub mod components;
mod new_project;
mod player;
mod server;

use anyhow::Context;
use kiwi_physics::physx::PhysicsKey;
use log::LevelFilter;
use player::PlayerRawInputHandler;
use server::QUIC_INTERFACE_PORT;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Set the path to the project. Defaults to the current directory
    #[arg(long)]
    project_path: Option<String>,

    /// Provide a public address or ip to the instance, which will allow users to connect to this instance over the internet
    ///
    /// Defaults to localhost
    #[arg(long)]
    public_host: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Create a new Kiwi project
    New { name: Option<String> },
    /// Builds and runs the project locally
    Run {
        #[clap(short, long)]
        user_id: Option<String>,
    },
    /// Builds the project
    Build,
    /// Builds and runs the project in server only mode
    Serve,
    /// View an asset
    View {
        asset_path: String,
        #[clap(short, long)]
        user_id: Option<String>,
    },
    /// Join a multiplayer session
    Join {
        host: Option<String>,
        #[clap(short, long)]
        user_id: Option<String>,
    },
    /// Updates all WASM APIs with the core primitive components (not for users)
    #[cfg(not(feature = "production"))]
    UpdateInterfaceComponents,
}
impl Commands {
    /// Will this command build assets?
    fn should_build(&self) -> bool {
        match self {
            Commands::New { .. } => false,
            Commands::Run { .. } => true,
            Commands::Build => true,
            Commands::Serve => true,
            Commands::View { .. } => true,
            Commands::Join { .. } => false,
            #[cfg(not(feature = "production"))]
            Commands::UpdateInterfaceComponents => false,
        }
    }
    /// Will this client run the application?
    fn should_run(&self) -> bool {
        match self {
            Commands::New { .. } => false,
            Commands::Run { .. } => true,
            Commands::Build => false,
            Commands::Serve => false,
            Commands::View { .. } => true,
            Commands::Join { .. } => true,
            #[cfg(not(feature = "production"))]
            Commands::UpdateInterfaceComponents => false,
        }
    }
    /// Will this join an external server?
    fn should_join(&self) -> bool {
        matches!(self, Commands::Join { .. })
    }
    fn user_id(&self) -> Option<&str> {
        match self {
            Commands::New { .. } => None,
            Commands::Run { user_id, .. } => user_id.as_deref(),
            Commands::Build => None,
            Commands::Serve => None,
            Commands::View { user_id, .. } => user_id.as_deref(),
            Commands::Join { user_id, .. } => user_id.as_deref(),
            #[cfg(not(feature = "production"))]
            Commands::UpdateInterfaceComponents => None,
        }
    }
}

fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![
            Box::new(kiwi_decals::client_systems()),
            Box::new(kiwi_primitives::systems()),
            Box::new(kiwi_sky::systems()),
            Box::new(kiwi_water::systems()),
            Box::new(kiwi_physics::client_systems()),
            Box::new(player::client_systems()),
        ],
    )
}

#[element_component]
fn GameView(hooks: &mut Hooks) -> Element {
    let (state, _) = hooks.consume_context::<GameClient>().unwrap();
    let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();

    let show_debug = true;
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
fn MainApp(hooks: &mut Hooks, server_addr: SocketAddr, user_id: String) -> Element {
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
                world.add_resource(kiwi_network::events::event_registry(), Arc::new(ServerEventRegistry::new()));
            }))),
            on_loaded: cb(move |_game_state, _game_client| Ok(Box::new(|| {}))),
            error_view: cb(move |error| Dock(vec![Text::el("Error").header_style(), Text::el(error)]).el()),
            systems_and_resources: cb(|| (client_systems(), EntityData::new())),
            create_rpc_registry: cb(server::create_rpc_registry),
            on_in_entities: None,
            ui: GameView.el(),
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
                    "kiwi_build",
                    "kiwi_gpu",
                    "kiwi_model",
                    "kiwi_network",
                    "kiwi_physics",
                    "kiwi_std",
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

    let cli = Cli::parse();

    let current_dir = std::env::current_dir()?;
    let project_path = cli.project_path.clone().map(|x| x.into()).unwrap_or_else(|| current_dir.clone());
    let project_path = if project_path.is_absolute() { project_path } else { kiwi_std::path::normalize(&current_dir.join(project_path)) };

    if project_path.exists() && !project_path.is_dir() {
        anyhow::bail!("Project path {project_path:?} exists and is not a directory.");
    }

    if let Commands::New { name } = cli.command {
        if let Err(err) = new_project::new_project(&project_path, name.as_deref()) {
            eprintln!("Failed to create project: {err:?}");
        }
        return Ok(());
    }

    #[cfg(not(feature = "production"))]
    if let Commands::UpdateInterfaceComponents = cli.command {
        let toml = components::dev::build_components_toml().to_string();

        // Assume we are being run within the codebase.
        for guest_path in std::fs::read_dir("guest/").unwrap().filter_map(Result::ok).map(|de| de.path()).filter(|de| de.is_dir()) {
            let toml_path = guest_path.join("api").join("kiwi.toml");
            std::fs::write(&toml_path, &toml)?;
            log::info!("Interface updated at {toml_path:?}");
        }
        return Ok(());
    }

    let manifest = if !cli.command.should_join() {
        let contents =
            std::fs::read_to_string(project_path.join("kiwi.toml")).context("No project manifest was found. Please create one.")?;
        Some(kiwi_project::Manifest::parse(&contents)?)
    } else {
        None
    };

    if cli.command.should_build() {
        runtime.block_on(kiwi_build::build(
            PhysicsKey.get(&assets),
            &assets,
            project_path.clone(),
            manifest.as_ref().expect("no manifest"),
        ));
    }

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
    let user_id = cli.command.user_id().map(|x| x.to_string()).unwrap_or_else(|| format!("user_{}", friendly_id()));
    let handle = runtime.handle().clone();
    if cli.command.should_run() {
        AppBuilder::simple().ui_renderer(true).with_runtime(runtime).with_asset_cache(assets).run(|app, _runtime| {
            MainApp { server_addr, user_id }.el().spawn_interactive(&mut app.world);
        });
    } else {
        handle.block_on(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {}
                Err(err) => log::error!("Unable to listen for shutdown signal: {}", err),
            }
        });
    }
    Ok(())
}
