use std::collections::HashMap;

use ambient_cameras::UICamera;
use ambient_ecs::{Entity, SystemGroup};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::{server::RpcArgs, web::client::GameClientView};
use ambient_rpc::RpcRegistry;
use ambient_std::friendly_id;
use ambient_ui_native::{cb, space_between_items, Dock, FlowColumn, StylesExt, Text};
use url::Url;

use crate::game_view::GameView;

#[element_component]
pub fn MainApp(hooks: &mut Hooks) -> Element {
    let url = Url::parse("https://127.0.0.1:9000").unwrap();

    FlowColumn(vec![
        Text::el(format!("Url: {url:?}")).header_style(),
        GameClientView {
            url,
            user_id: friendly_id(),
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
            on_loaded: cb(move |client| {
                let mut game_state = client.game_state.lock();
                let world = &mut game_state.world;

                // TODO: client side wasm on the web
                // wasm::initialize(world).unwrap();

                UICamera.el().spawn_static(world);

                Ok(Box::new(|| {
                    tracing::info!("Disconnecting client");
                }))
            }),
            create_rpc_registry: cb(create_server_rpc_registry),
            inner: Dock::el(vec![GameView { show_debug: false }.el()]),
        }
        .el(),
    ])
    .el()
    .with(space_between_items(), 10.)
}

/// Declares the systems to run in the network client world
fn systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![
            // Box::new(ambient_prefab::systems()),
            // Box::new(ambient_decals::client_systems()),
            // Box::new(ambient_primitives::systems()),
            // Box::new(ambient_sky::systems()),
            // Box::new(ambient_water::systems()),
            // Box::new(ambient_physics::client_systems()),
            // Box::new(wasm::systems()),
            // Box::new(player::systems_final()),
        ],
    )
}

pub fn create_server_rpc_registry() -> RpcRegistry<RpcArgs> {
    let mut reg = RpcRegistry::new();
    ambient_network::rpc::register_server_rpcs(&mut reg);
    // ambient_debugger::register_server_rpcs(&mut reg);
    reg
}
