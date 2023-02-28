use std::{net::SocketAddr, sync::Arc};

use ambient_app::{window_title, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_debugger::Debugger;
use ambient_ecs::{Entity, SystemGroup};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::{
    client::{GameClient, GameClientNetworkStats, GameClientRenderTarget, GameClientServerStats, GameClientView, UseOnce},
    events::ServerEventRegistry,
};
use ambient_std::{asset_cache::AssetCache, cb};
use ambient_ui::{use_window_physical_resolution, Dock, FocusRoot, StylesExt, Text, WindowSized};

use crate::shared;

/// Construct an app and enter the main client view
pub async fn run(assets: AssetCache, server_addr: SocketAddr, user_id: String, show_debug: bool) {
    AppBuilder::simple()
        .ui_renderer(true)
        .with_asset_cache(assets)
        .run(|app, _runtime| {
            MainApp { server_addr, user_id, show_debug }.el().spawn_interactive(&mut app.world);
        })
        .await;
}

#[element_component]
fn MainApp(hooks: &mut Hooks, server_addr: SocketAddr, user_id: String, show_debug: bool) -> Element {
    let resolution = use_window_physical_resolution(hooks);

    hooks.provide_context(GameClientNetworkStats::default);
    hooks.provide_context(GameClientServerStats::default);

    *hooks.world.resource_mut(window_title()) = "Ambient".to_string();

    FocusRoot::el([
        UICamera.el().set(active_camera(), 0.),
        shared::player::PlayerRawInputHandler.el(),
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
            systems_and_resources: cb(|| (systems(), Entity::new())),
            create_rpc_registry: cb(shared::create_rpc_registry),
            on_in_entities: None,
            ui: GameView { show_debug }.el(),
        }
        .el()]),
    ])
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
            Box::new(shared::player::client_systems()),
        ],
    )
}
