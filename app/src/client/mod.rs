use std::{net::SocketAddr, path::PathBuf, process::exit, sync::Arc, time::Duration};

use ambient_app::{fps_stats, window_title, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::{
    runtime,
    window::{window_ctl, WindowCtl},
};
use ambient_debugger::Debugger;
use ambient_ecs::{Entity, SystemGroup};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::{
    client::{GameClient, GameClientNetworkStats, GameClientRenderTarget, GameClientServerStats, GameClientView, UseOnce},
    events::ServerEventRegistry,
};
use ambient_std::{asset_cache::AssetCache, cb, friendly_id};
use ambient_ui::{use_window_physical_resolution, Dock, FocusRoot, StylesExt, Text, WindowSized};
use glam::uvec2;

use crate::{cli::RunCli, shared};

mod wasm;

/// Construct an app and enter the main client view
pub async fn run(assets: AssetCache, server_addr: SocketAddr, run: &RunCli, project_path: Option<PathBuf>) {
    let user_id = run.user_id.clone().unwrap_or_else(|| format!("user_{}", friendly_id()));
    let headless = if run.headless { Some(uvec2(600, 600)) } else { None };

    let is_debug = std::env::var("AMBIENT_DEBUGGER").is_ok() || run.debugger;

    AppBuilder::new()
        .ui_renderer(true)
        .with_asset_cache(assets)
        .headless(headless)
        .update_title_with_fps_stats(false)
        .run(move |app, _runtime| {
            *app.world.resource_mut(window_title()) = "Ambient".to_string();
            MainApp { server_addr, user_id, show_debug: is_debug, golden_image_test: run.golden_image_test, project_path }
                .el()
                .spawn_interactive(&mut app.world);
        })
        .await;
}

#[element_component]
fn TitleUpdater(hooks: &mut Hooks) -> Element {
    let net = hooks.consume_context::<GameClientNetworkStats>().map(|stats| stats.0);
    let world = &hooks.world;
    let title = world.resource(window_title());
    let fps = world.get_cloned(hooks.world.resource_entity(), fps_stats()).ok().filter(|f| !f.fps().is_nan());

    let title = match (fps, net) {
        (None, None) => title.clone(),
        (Some(fps), None) => format!("{} [{}]", title, fps.dump_both()),
        (None, Some(net)) => format!("{} [{}]", title, net),
        (Some(fps), Some(net)) => format!("{} [{}, {}]", title, fps.dump_both(), net),
    };
    world.resource(window_ctl()).send(WindowCtl::SetTitle(title)).ok();

    Element::new()
}

#[element_component]
fn MainApp(
    hooks: &mut Hooks,
    server_addr: SocketAddr,
    project_path: Option<PathBuf>,
    user_id: String,
    show_debug: bool,
    golden_image_test: Option<f32>,
) -> Element {
    let resolution = use_window_physical_resolution(hooks);

    let update_network_stats = hooks.provide_context(GameClientNetworkStats::default);
    let update_server_stats = hooks.provide_context(GameClientServerStats::default);

    FocusRoot::el([
        UICamera.el(),
        shared::player::PlayerRawInputHandler.el(),
        shared::player::PlayerDataUpload.el(),
        TitleUpdater.el(),
        WindowSized::el([GameClientView {
            server_addr,
            user_id,
            resolution,
            on_disconnect: cb(move || {}),
            init_world: cb(UseOnce::new(Box::new(move |world, _render_target| {
                wasm::initialize(world).unwrap();

                world.add_resource(ambient_network::events::event_registry(), Arc::new(ServerEventRegistry::new()));
                UICamera.el().spawn_static(world);
            }))),
            on_loaded: cb(move |_game_state, _game_client| Ok(Box::new(|| {}))),
            error_view: cb(move |error| Dock(vec![Text::el("Error").header_style(), Text::el(error)]).el()),
            on_network_stats: cb(move |stats| update_network_stats(stats)),
            on_server_stats: cb(move |stats| update_server_stats(stats)),
            systems_and_resources: cb(|| (systems(), Entity::new())),
            create_rpc_registry: cb(shared::create_rpc_registry),
            on_in_entities: None,
            ui: Dock::el(vec![GoldenImageTest::el(project_path, golden_image_test), GameView { show_debug }.el()]),
        }
        .el()]),
    ])
}

#[element_component]
fn GoldenImageTest(hooks: &mut Hooks, project_path: Option<PathBuf>, golden_image_test: Option<f32>) -> Element {
    let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();
    hooks.use_spawn(move |world| {
        if let Some(seconds) = golden_image_test {
            world.resource(runtime()).spawn(async move {
                tokio::time::sleep(Duration::from_secs_f32(seconds)).await;
                let screenshot = project_path.unwrap_or(PathBuf::new()).join("screenshot.png");
                log::info!("Loading screenshot from {:?}", screenshot);
                let old = image::open(&screenshot);
                log::info!("Saving screenshot to {:?}", screenshot);
                let new = render_target.0.color_buffer.reader().read_image().await.unwrap().into_rgba8();
                log::info!("Screenshot saved");
                new.save(screenshot).unwrap();

                if let Ok(old) = old {
                    log::info!("Comparing screenshots");

                    let hasher = image_hasher::HasherConfig::new().to_hasher();

                    let hash1 = hasher.hash_image(&new);
                    let hash2 = hasher.hash_image(&old);
                    let dist = hash1.dist(&hash2);
                    if dist > 0 {
                        log::info!("Screenshots differ, distance={}", dist);
                        exit(1);
                    } else {
                        log::info!("Screenshots are identical");
                        exit(0);
                    }
                } else {
                    log::info!("No old screenshot to compare to");
                    exit(1);
                }
            });
        }
        Box::new(|_| {})
    });
    Element::new()
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
