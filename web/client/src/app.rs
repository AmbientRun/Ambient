use crate::wasm;
use ambient_cameras::UICamera;
use ambient_client_shared::{game_view::GameView, player};
use ambient_ecs::{Entity, SystemGroup};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::{server::RpcArgs, web::client::GameClientView};
use ambient_rpc::RpcRegistry;
use ambient_ui_native::cb;
use std::collections::HashMap;

#[element_component]
pub fn MainApp(_hooks: &mut Hooks, server_url: String, fail_on_version_mismatch: bool) -> Element {
    tracing::info!("Connecting to {server_url:?}");

    GameClientView {
        url: server_url,
        user_id: ambient_client_shared::util::random_username(),
        fail_on_version_mismatch,
        systems_and_resources: cb(|| {
            let mut resources = Entity::new();

            let bistream_handlers = HashMap::new();
            resources.set(
                ambient_network::client::bi_stream_handlers(),
                bistream_handlers,
            );

            let unistream_handlers = HashMap::new();
            resources.set(
                ambient_network::client::uni_stream_handlers(),
                unistream_handlers,
            );

            let dgram_handlers = HashMap::new();
            resources.set(ambient_network::client::datagram_handlers(), dgram_handlers);

            (systems(), resources)
        }),
        on_loaded: cb(move |_, game_state| {
            let world = &mut game_state.world;

            wasm::initialize(world).unwrap();

            UICamera.el().spawn_static(world);

            Ok(Box::new(|| {
                tracing::info!("Disconnecting client");
            }))
        }),
        create_rpc_registry: cb(create_server_rpc_registry),
        inner: GameView { show_debug: false }.el(),
    }
    .el()
}

/// Declares the systems to run in the network client world
fn systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![
            Box::new(ambient_prefab::systems()),
            // Box::new(ambient_decals::client_systems()),
            Box::new(ambient_primitives::systems()),
            // Box::new(ambient_sky::systems()),
            // Box::new(ambient_water::systems()),
            // Box::new(ambient_physics::client_systems()),
            // Box::new(ambient_gizmos::client_systems()),
            Box::new(wasm::systems()),
            Box::new(player::systems_final()),
        ],
    )
}

pub fn create_server_rpc_registry() -> RpcRegistry<RpcArgs> {
    let mut reg = RpcRegistry::new();
    ambient_network::rpc::register_server_rpcs(&mut reg);
    // ambient_debugger::register_server_rpcs(&mut reg);
    reg
}
