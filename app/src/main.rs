use std::{net::SocketAddr, sync::Arc};

use clap::{Parser, Subcommand};
use elements_app::{AppBuilder, ExamplesSystem};
use elements_cameras::UICamera;
use elements_core::camera::active_camera;
use elements_debugger::RendererDebugger;
use elements_ecs::{EntityData, SystemGroup, World};
use elements_element::{element_component, Element, ElementComponentExt, Hooks};
use elements_network::{
    client::{GameClient, GameClientNetworkStats, GameClientRenderTarget, GameClientServerStats, GameClientView, UseOnce}, events::ServerEventRegistry
};
use elements_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt}, Cb
};
use elements_ui::{use_window_physical_resolution, Dock, FocusRoot, StylesExt, Text, WindowSized};

pub mod components;
mod new_project;
mod player;
mod server;

use anyhow::Context;
use elements_physics::physx::PhysicsKey;
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
    /// Defaults to localhost
    #[arg(long)]
    public_host: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Create a new Elements project
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
        }
    }
}

fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![Box::new(elements_decals::client_systems()), Box::new(elements_primitives::systems()), Box::new(player::client_systems())],
    )
}

#[element_component]
fn GameView(_world: &mut World, hooks: &mut Hooks) -> Element {
    let (state, _) = hooks.consume_context::<GameClient>().unwrap();
    let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();

    let show_debug = true;
    if show_debug {
        RendererDebugger {
            get_state: Cb::new(move |cb| {
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
fn MainApp(world: &mut World, hooks: &mut Hooks, server_addr: SocketAddr, user_id: String) -> Element {
    let resolution = use_window_physical_resolution(world, hooks);

    hooks.provide_context(GameClientNetworkStats::default);
    hooks.provide_context(GameClientServerStats::default);

    FocusRoot::el([
        UICamera.el().set(active_camera(), 0.),
        PlayerRawInputHandler.el(),
        WindowSized::el([GameClientView {
            server_addr,
            user_id,
            resolution,
            on_disconnect: Cb::new(move || {}),
            init_world: Cb::new(UseOnce::new(Box::new(move |world, _render_target| {
                world.add_resource(elements_network::events::event_registry(), Arc::new(ServerEventRegistry::new()));
            }))),
            on_loaded: Cb::new(move |_game_state, _game_client| Ok(Box::new(|| {}))),
            error_view: Cb(Arc::new(move |error| Dock(vec![Text::el("Error").header_style(), Text::el(error)]).el())),
            systems_and_resources: Cb::new(|| (client_systems(), EntityData::new())),
            create_rpc_registry: Cb::new(server::create_rpc_registry),
            on_in_entities: None,
            ui: GameView.el(),
        }
        .el()]),
    ])
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    components::init()?;
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    let assets = AssetCache::new(runtime.handle().clone());
    PhysicsKey.get(&assets); // Load physics

    let cli = Cli::parse();

    let current_dir = std::env::current_dir()?;
    let project_path = cli.project_path.clone().map(|x| x.into()).unwrap_or_else(|| current_dir.clone());
    let project_path =
        if project_path.is_absolute() { project_path } else { elements_std::path::normalize(&current_dir.join(project_path)) };

    if project_path.exists() && !project_path.is_dir() {
        anyhow::bail!("Project path {project_path:?} exists and is not a directory.");
    }

    if let Commands::New { name } = cli.command {
        if let Err(err) = new_project::new_project(&project_path, name.as_deref()) {
            eprintln!("Failed to create project: {err:?}");
        }
        return Ok(());
    }

    let manifest = if !cli.command.should_join() {
        let contents =
            std::fs::read_to_string(project_path.join("elements.toml")).context("No project manifest was found. Please create one.")?;
        Some(elements_project::Manifest::parse(&contents)?)
    } else {
        None
    };

    if cli.command.should_build() {
        runtime.block_on(elements_build::build(
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
        eprintln!("Server running on port {port}");
        format!("127.0.0.1:{port}").parse()?
    };
    let user_id = cli.command.user_id().map(|x| x.to_string()).unwrap_or_else(|| format!("user_{}", friendly_id::create()));
    let handle = runtime.handle().clone();
    if cli.command.should_run() {
        AppBuilder::simple().ui_renderer(true).with_runtime(runtime).with_asset_cache(assets).run(|app, _runtime| {
            app.window_event_systems.add(Box::new(ExamplesSystem));
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
