use std::collections::HashMap;

use ambient_cameras::UICamera;
use ambient_client_shared::player;
use ambient_ecs::{Entity, SystemGroup};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_network::{server::RpcArgs, web::client::GameClientView};
use ambient_rpc::RpcRegistry;
use ambient_std::friendly_id;
use ambient_ui_native::{cb, space_between_items, Dock, FlowColumn, Text};
use url::Url;

use crate::game_view::GameView;

#[element_component]
pub fn MainApp(_hooks: &mut Hooks) -> Element {
    let url = Url::parse("https://127.0.0.1:9000").unwrap();

    // FlowColumn(vec![
        // Button::new("Dump native UI", |w| {
        //     let mut buf = Vec::new();
        //     dump_world_hierarchy(w, &mut buf);
        //     let s = String::from_utf8(buf).unwrap();
        //
        //     tracing::info!("Dumping native UI: {}", s.len());
        //     ambient_sys::task::RuntimeHandle::current().spawn_local({
        //         async move {
        //             let s = s;
        //             ambient_sys::clipboard::set(&s).await;
        //         }
        //     });
        // })
        // .el(),
        // Text::el(format!("Url: {url}")),
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
        .el()
    // ])
    // .el()
    // .with(space_between_items(), 10.)
}

/// Declares the systems to run in the network client world
fn systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![
            // Box::new(ambient_prefab::systems()),
            // Box::new(ambient_decals::client_systems()),
            Box::new(ambient_primitives::systems()),
            // Box::new(ambient_sky::systems()),
            // Box::new(ambient_water::systems()),
            // Box::new(ambient_physics::client_systems()),
            // Box::new(wasm::systems()),
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
