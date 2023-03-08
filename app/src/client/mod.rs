use std::{net::SocketAddr, path::PathBuf, process::exit, sync::Arc, time::Duration};

use ambient_app::{window_title, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::{camera::active_camera, runtime};
use ambient_debugger::Debugger;
use ambient_ecs::{Entity, SystemGroup, World};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::{
    client::{GameClient, GameClientNetworkStats, GameClientRenderTarget, GameClientServerStats, GameClientView, UseOnce},
    events::ServerEventRegistry,
};
use ambient_std::{asset_cache::AssetCache, cb, friendly_id};
use ambient_ui::{use_window_physical_resolution, Dock, FocusRoot, StylesExt, Text, WindowSized};

use crate::{cli::RunCli, shared};
use ambient_renderer::RenderTarget;
use glam::uvec2;

mod wasm;

/// Construct an app and enter the main client view
pub async fn run(assets: AssetCache, server_addr: SocketAddr, run: &RunCli, project_path: Option<PathBuf>) {
    let user_id = run.user_id.clone().unwrap_or_else(|| format!("user_{}", friendly_id()));
    let headless = if run.headless { Some(uvec2(400, 400)) } else { None };

    let is_debug = std::env::var("AMBIENT_DEBUGGER").is_ok() || run.debugger;

    AppBuilder::simple()
        .ui_renderer(true)
        .with_asset_cache(assets)
        .headless(headless)
        .run(move |app, _runtime| {
            MainApp { server_addr, user_id, show_debug: is_debug, screenshot_test: run.screenshot_test, project_path }
                .el()
                .spawn_interactive(&mut app.world);
        })
        .await;
}

#[element_component]
fn MainApp(
    hooks: &mut Hooks,
    server_addr: SocketAddr,
    project_path: Option<PathBuf>,
    user_id: String,
    show_debug: bool,
    screenshot_test: Option<f32>,
) -> Element {
    let resolution = use_window_physical_resolution(hooks);

    let update_network_stats = hooks.provide_context(GameClientNetworkStats::default);
    let update_server_stats = hooks.provide_context(GameClientServerStats::default);

    *hooks.world.resource_mut(window_title()) = "Ambient".to_string();

    FocusRoot::el([
        UICamera.el().set(active_camera(), 0.),
        shared::player::PlayerRawInputHandler.el(),
        shared::player::PlayerDataUpload.el(),
        WindowSized::el([GameClientView {
            server_addr,
            user_id,
            resolution,
            on_disconnect: cb(move || {}),
            init_world: cb(UseOnce::new(Box::new(move |world, render_target| {
                wasm::initialize(world).unwrap();

                world.add_resource(ambient_network::events::event_registry(), Arc::new(ServerEventRegistry::new()));
                if let Some(seconds) = screenshot_test {
                    run_screenshot_test(world, render_target, project_path, seconds);
                }
            }))),
            on_loaded: cb(move |_game_state, _game_client| Ok(Box::new(|| {}))),
            error_view: cb(move |error| Dock(vec![Text::el("Error").header_style(), Text::el(error)]).el()),
            on_network_stats: cb(move |stats| update_network_stats(stats)),
            on_server_stats: cb(move |stats| update_server_stats(stats)),
            systems_and_resources: cb(|| (systems(), Entity::new())),
            create_rpc_registry: cb(shared::create_rpc_registry),
            on_in_entities: None,
            ui: GameView { show_debug }.el(),
        }
        .el()]),
    ])
}

fn run_screenshot_test(world: &World, render_target: Arc<RenderTarget>, project_path: Option<PathBuf>, seconds: f32) {
    world.resource(runtime()).spawn(async move {
        tokio::time::sleep(Duration::from_secs_f32(seconds)).await;
        let screenshot = project_path.unwrap_or(PathBuf::new()).join("screenshot.png");
        log::info!("Loading screenshot from {:?}", screenshot);
        let old = image::open(&screenshot);
        log::info!("Saving screenshot to {:?}", screenshot);
        let new = render_target.color_buffer.reader().read_image().await.unwrap().into_rgba8();
        log::info!("Screenshot saved");
        new.save(screenshot).unwrap();
        let epsilon = 3;
        if let Ok(old) = old {
            log::info!("Comparing screenshots");
            let old = old.into_rgba8();
            for (a, b) in old.pixels().zip(new.pixels()) {
                if (a[0]).abs_diff(b[0]) > epsilon
                    || (a[1]).abs_diff(b[1]) > epsilon
                    || (a[2]).abs_diff(b[2]) > epsilon
                    || (a[3]).abs_diff(b[3]) > epsilon
                {
                    log::info!("Screenshots differ");
                    exit(1);
                }
            }
            log::info!("Screenshots are identical");
            exit(0);
        } else {
            log::info!("No old screenshot to compare to");
            exit(1);
        }
    });
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

fn systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![
            Box::new(ambient_decals::client_systems()),
            Box::new(ambient_primitives::systems()),
            Box::new(ambient_sky::systems()),
            Box::new(ambient_water::systems()),
            Box::new(ambient_physics::client_systems()),
            Box::new(wasm::systems()),
        ],
    )
}
