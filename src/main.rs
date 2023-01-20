use std::sync::Arc;

use clap::{Parser, Subcommand};
use elements_app::{AppBuilder, ExamplesSystem};
use elements_cameras::UICamera;
use elements_core::{camera::active_camera, main_scene};
use elements_ecs::{EntityData, SystemGroup, World};
use elements_element::{element_component, Element, ElementComponentExt, Hooks};
use elements_network::{
    client::{GameClient, GameClientNetworkStats, GameClientRenderTarget, GameClientServerStats, GameClientView, UseOnce}, client_game_state::game_screen_render_target, events::ServerEventRegistry
};
use elements_renderer_debugger::RendererDebugger;
use elements_std::{asset_cache::AssetCache, math::SphericalCoords, Cb};
use elements_ui::{use_window_logical_resolution, use_window_physical_resolution, Dock, FocusRoot, StylesExt, Text, WindowSized};
use glam::vec3;

pub mod scripting;
mod server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Set the path to the project. Defaults to the current directory
    #[arg(short, long)]
    project_path: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Builds and runs the project
    Run,
    /// Builds the project
    Build,
    /// View an asset
    View { asset_path: String },
}
impl Commands {
    fn should_build(&self) -> bool {
        match self {
            Commands::Run => true,
            Commands::Build => true,
            Commands::View { .. } => true,
        }
    }
    fn should_run(&self) -> bool {
        match self {
            Commands::Run => true,
            Commands::Build => false,
            Commands::View { .. } => true,
        }
    }
}

fn client_systems() -> SystemGroup {
    SystemGroup::new("client", vec![Box::new(elements_decals::client_systems())])
}

#[element_component]
fn GameView(world: &mut World, hooks: &mut Hooks) -> Element {
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
fn MainApp(world: &mut World, hooks: &mut Hooks, port: u16) -> Element {
    let screen_size = use_window_logical_resolution(world, hooks);
    let resolution = use_window_physical_resolution(world, hooks);

    hooks.provide_context(GameClientNetworkStats::default);
    hooks.provide_context(GameClientServerStats::default);

    FocusRoot::el([
        UICamera.el().set(active_camera(), 0.),
        WindowSized::el([GameClientView {
            server_addr: format!("127.0.0.1:{port}").parse().unwrap(),
            user_id: "host".to_string(),
            resolution,
            on_disconnect: Cb::new(move || {}),
            init_world: Cb::new(UseOnce::new(Box::new(move |world, _render_target| {
                world.add_resource(elements_network::events::event_registry(), Arc::new(ServerEventRegistry::new()));
                // Cube.el().spawn_static(world);
                // Quad.el().set(scale(), Vec3::ONE * 10.).spawn_static(world);

                elements_cameras::spherical::new(
                    vec3(0., 0., 0.),
                    SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.),
                )
                .set(active_camera(), 0.)
                .set(main_scene(), ())
                .spawn(world);
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

fn init_all_components() {
    elements_app::init_all_components();
    elements_network::init_all_components();
    elements_physics::init_all_components();
    elements_scripting_host::shared::init_components();
    elements_decals::init_components();
}

fn main() {
    env_logger::init();
    init_all_components();
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let assets = AssetCache::new(runtime.handle().clone());

    let cli = Cli::parse();
    let current_dir = std::env::current_dir().unwrap();
    let project_path = cli.project_path.clone().map(|x| x.into()).unwrap_or_else(|| current_dir.clone());

    let project_path =
        if project_path.is_absolute() { project_path } else { elements_std::path::normalize(&current_dir.join(project_path)) };

    if cli.command.should_build() {
        runtime.block_on(elements_build::build(&assets, project_path.clone()));
    }

    if cli.command.should_run() {
        let port = server::start_server(&runtime, assets.clone(), cli, project_path);
        AppBuilder::simple().ui_renderer(true).with_runtime(runtime).with_asset_cache(assets).run(|app, _runtime| {
            app.window_event_systems.add(Box::new(ExamplesSystem));
            MainApp { port }.el().spawn_interactive(&mut app.world);
        });
    }
}
