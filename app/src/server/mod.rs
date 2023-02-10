use std::{
    collections::HashMap, net::SocketAddr, path::{Path, PathBuf}, sync::Arc, time::SystemTime
};

use anyhow::Context;
use axum::{
    http::{Method, StatusCode}, response::IntoResponse, routing::{get, get_service}, Router
};
use kiwi_core::{app_start_time, asset_cache, dtime, no_sync, time};
use kiwi_ecs::{ComponentDesc, ComponentRegistry, EntityData, Networked, SystemGroup, World, WorldStreamCompEvent};
use kiwi_network::{
    bi_stream_handlers, client::GameRpcArgs, datagram_handlers, server::{ForkingEvent, GameServer, ShutdownEvent}
};
use kiwi_object::ObjectFromUrl;
use kiwi_rpc::RpcRegistry;
use kiwi_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKeyExt}, asset_url::{AbsAssetUrl, ServerBaseUrlKey}
};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{player, Cli, Commands};

mod scripting;

fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "server",
        vec![
            kiwi_physics::fetch_simulation_system(),
            Box::new(kiwi_physics::physx::sync_ecs_physics()),
            Box::new(kiwi_core::async_ecs::async_ecs_systems()),
            Box::new(kiwi_core::transform::TransformSystem::new()),
            kiwi_core::remove_at_time_system(),
            Box::new(kiwi_physics::physics_server_systems()),
            Box::new(player::server_systems()),
            Box::new(scripting::systems()),
            Box::new(player::server_systems_final()),
            kiwi_physics::run_simulation_system(),
        ],
    )
}
fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    SystemGroup::new("on_forking_systems", vec![Box::new(kiwi_physics::on_forking_systems()), Box::new(scripting::on_forking_systems())])
}
fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    SystemGroup::new("on_shutdown_systems", vec![Box::new(kiwi_physics::on_shutdown_systems()), Box::new(scripting::on_shutdown_systems())])
}

fn is_sync_component(component: ComponentDesc, _: WorldStreamCompEvent) -> bool {
    component.has_attribute::<Networked>()
}

pub fn create_rpc_registry() -> RpcRegistry<GameRpcArgs> {
    let mut reg = RpcRegistry::new();
    kiwi_network::rpc::register_rpcs(&mut reg);
    kiwi_debugger::register_rpcs(&mut reg);
    reg
}

fn create_server_resources(assets: AssetCache) -> EntityData {
    let mut server_resources = EntityData::new().set(asset_cache(), assets.clone()).set(no_sync(), ());

    kiwi_physics::create_server_resources(&assets, &mut server_resources);

    server_resources.append_self(kiwi_core::async_ecs::async_ecs_resources());
    server_resources.set_self(kiwi_core::runtime(), tokio::runtime::Handle::current());
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    server_resources.set_self(time(), now);
    server_resources.set_self(app_start_time(), now);
    server_resources.set_self(dtime(), 1. / 60.);

    let mut handlers = HashMap::new();
    kiwi_network::register_rpc_bi_stream_handler(&mut handlers, create_rpc_registry());
    server_resources.set_self(bi_stream_handlers(), handlers);

    let mut handlers = HashMap::new();
    player::register_datagram_handler(&mut handlers);
    server_resources.set_self(datagram_handlers(), handlers);

    server_resources
}

pub const HTTP_INTERFACE_PORT: u16 = 8999;
pub const QUIC_INTERFACE_PORT: u16 = 9000;

fn start_http_interface(runtime: &tokio::runtime::Runtime, project_path: &Path) {
    let router = Router::new()
        .route("/ping", get(|| async move { "ok" }))
        .route("/assets", get_service(ServeDir::new(project_path.join("target"))).handle_error(handle_error))
        .layer(CorsLayer::new().allow_origin(tower_http::cors::Any).allow_methods(vec![Method::GET]).allow_headers(tower_http::cors::Any));

    runtime.spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], HTTP_INTERFACE_PORT));
        axum::Server::bind(&addr).serve(router.into_make_service()).await.unwrap();
    });
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

pub(crate) fn start_server(
    runtime: &tokio::runtime::Runtime,
    assets: AssetCache,
    cli: Cli,
    project_path: PathBuf,
    manifest: &kiwi_project::Manifest,
) -> u16 {
    log::info!("Creating server");
    let server = runtime.block_on(async move {
        GameServer::new_with_port_in_range(QUIC_INTERFACE_PORT..(QUIC_INTERFACE_PORT + 10))
            .await
            .context("failed to create game server with port in range")
            .unwrap()
    });
    let port = server.port;

    scripting::init_all_components();
    let public_host =
        cli.public_host.or_else(|| local_ip_address::local_ip().ok().map(|x| x.to_string())).unwrap_or("localhost".to_string());
    log::info!("Created server, running at {public_host}:{port}");
    ServerBaseUrlKey.insert(&assets, AbsAssetUrl::parse(format!("http://{public_host}:{HTTP_INTERFACE_PORT}/assets/")).unwrap());

    start_http_interface(runtime, &project_path);

    ComponentRegistry::get_mut().add_external(manifest.all_defined_components(false).unwrap());

    let manifest = manifest.clone();
    runtime.spawn(async move {
        let mut server_world = World::new_with_config("server", true);
        server_world.init_shape_change_tracking();

        server_world.add_components(server_world.resource_entity(), create_server_resources(assets.clone())).unwrap();

        scripting::initialize(&mut server_world, project_path.clone(), &manifest).await.unwrap();

        if let Commands::View { asset_path, .. } = cli.command.clone() {
            let asset_path = AbsAssetUrl::from_file_path(project_path.join("target").join(asset_path).join("objects/main.json"));
            log::info!("Spawning asset from {:?}", asset_path);
            let obj = ObjectFromUrl(asset_path).get(&assets).await.unwrap();
            obj.spawn_into_world(&mut server_world, None);
        }
        log::info!("Starting server");
        server
            .run(
                server_world,
                Arc::new(|_world| server_systems()),
                Arc::new(|| on_forking_systems()),
                Arc::new(|| on_shutdown_systems()),
                Arc::new(is_sync_component),
            )
            .await;
    });
    port
}
