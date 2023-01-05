use std::{collections::HashMap, sync::Arc, time::SystemTime};

use anyhow::Context;
use elements_core::{app_start_time, asset_cache, dtime, remove_at_time, time};
use elements_ecs::{EntityData, SystemGroup, World, WorldStreamCompEvent};
use elements_network::{
    bi_stream_handlers, client::GameRpcArgs, datagram_handlers, server::{ForkingEvent, GameServer, ShutdownEvent}
};
use elements_rpc::RpcRegistry;
use elements_std::asset_cache::AssetCache;

fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "server",
        vec![
            Box::new(elements_core::async_ecs::async_ecs_systems()),
            Box::new(elements_core::transform::TransformSystem::new()),
            elements_core::remove_at_time_system(),
        ],
    )
}
fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    SystemGroup::new("on_forking_systems", vec![Box::new(elements_physics::on_forking_systems())])
}
fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    SystemGroup::new("on_shutdown_systems", vec![Box::new(elements_physics::on_shutdown_systems())])
}
fn is_sync_component(component: &dyn elements_ecs::IComponent, event: WorldStreamCompEvent) -> bool {
    let mut res = component.is_extended() && component.clone_boxed() != remove_at_time();

    res
}

pub fn create_rpc_registry() -> RpcRegistry<GameRpcArgs> {
    let mut reg = RpcRegistry::new();
    elements_network::rpc::register_rpcs(&mut reg);
    reg
}

fn create_server_resources(assets: AssetCache) -> EntityData {
    let mut server_resources = EntityData::new().set(asset_cache(), assets);

    server_resources.append_self(elements_core::async_ecs::async_ecs_resources());
    server_resources.set_self(elements_core::runtime(), tokio::runtime::Handle::current());
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    server_resources.set_self(time(), now);
    server_resources.set_self(app_start_time(), now);
    server_resources.set_self(dtime(), 1. / 60.);

    let mut handlers = HashMap::new();
    elements_network::register_rpc_bi_stream_handler(&mut handlers, create_rpc_registry());
    server_resources.set_self(bi_stream_handlers(), handlers);

    let mut handlers = HashMap::new();
    server_resources.set_self(datagram_handlers(), handlers);

    server_resources
}

pub fn start_server(runtime: &tokio::runtime::Runtime, assets: AssetCache) -> u16 {
    log::info!("Creating server");
    let server = runtime.block_on(async move {
        GameServer::new_with_port_in_range(9000..(9000 + 10)).await.context("failed to create game server with port in range").unwrap()
    });
    let port = server.port;
    log::info!("Server created on port {port}");
    runtime.spawn(async move {
        let mut server_world = World::new_with_config("server", 1, true);
        server_world.init_shape_change_tracking();

        server_world.add_components(server_world.resource_entity(), create_server_resources(assets)).unwrap();
        log::info!("Starting server");
        server
            .run(
                server_world,
                Arc::new(|world| server_systems()),
                Arc::new(|| on_forking_systems()),
                Arc::new(|| on_shutdown_systems()),
                Arc::new(is_sync_component),
            )
            .await;
    });
    port
}
