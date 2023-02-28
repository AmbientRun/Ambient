use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use ambient_core::{app_start_time, asset_cache, dtime, no_sync, project_name, time};
use ambient_ecs::{
    world_events, ComponentDesc, ComponentRegistry, Entity, Networked, SystemGroup, World, WorldEventsSystem, WorldStreamCompEvent,
};
use ambient_network::{
    bi_stream_handlers, datagram_handlers,
    server::{ForkingEvent, GameServer, ShutdownEvent},
};
use ambient_prefab::PrefabFromUrl;
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ServerBaseUrlKey},
};
use ambient_sys::task::RuntimeHandle;
use anyhow::Context;
use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{cli::Cli, shared};

mod wasm;

pub fn start(
    runtime: &tokio::runtime::Runtime,
    assets: AssetCache,
    cli: Cli,
    project_path: PathBuf,
    manifest: &ambient_project::Manifest,
) -> u16 {
    log::info!("Creating server");
    let server = runtime.block_on(async move {
        GameServer::new_with_port_in_range(QUIC_INTERFACE_PORT..(QUIC_INTERFACE_PORT + 10), false)
            .await
            .context("failed to create game server with port in range")
            .unwrap()
    });
    let port = server.port;

    wasm::init_all_components();
    let public_host = cli
        .host()
        .and_then(|h| h.public_host.clone())
        .or_else(|| local_ip_address::local_ip().ok().map(|x| x.to_string()))
        .unwrap_or("localhost".to_string());
    log::info!("Created server, running at {public_host}:{port}");
    ServerBaseUrlKey.insert(&assets, AbsAssetUrl::parse(format!("http://{public_host}:{HTTP_INTERFACE_PORT}/content/")).unwrap());

    start_http_interface(runtime, &project_path);

    ComponentRegistry::get_mut().add_external(manifest.all_defined_components(false).unwrap());

    let manifest = manifest.clone();
    runtime.spawn(async move {
        let mut server_world = World::new_with_config("server", true);
        server_world.init_shape_change_tracking();

        server_world.add_components(server_world.resource_entity(), create_resources(assets.clone())).unwrap();

        // Keep track of the project name
        let name = manifest.project.name.clone().unwrap_or_else(|| "Ambient".into());
        server_world.add_components(server_world.resource_entity(), Entity::new().with(project_name(), name)).unwrap();

        wasm::initialize(&mut server_world, project_path.clone(), &manifest).await.unwrap();

        if let Cli::View { asset_path, .. } = cli.clone() {
            let asset_path = AbsAssetUrl::from_file_path(project_path.join("build").join(asset_path).join("prefabs/main.json"));
            log::info!("Spawning asset from {:?}", asset_path);
            let obj = PrefabFromUrl(asset_path.into()).get(&assets).await.unwrap();
            obj.spawn_into_world(&mut server_world, None);
        }
        log::info!("Starting server");
        server
            .run(server_world, Arc::new(systems), Arc::new(on_forking_systems), Arc::new(on_shutdown_systems), Arc::new(is_sync_component))
            .await;
    });
    port
}

fn systems(_world: &mut World) -> SystemGroup {
    SystemGroup::new(
        "server",
        vec![
            ambient_physics::run_simulation_system(),
            // Can happen *during* the physics step
            Box::new(ambient_core::async_ecs::async_ecs_systems()),
            Box::new(ambient_prefab::systems()),
            // Happens after the physics step
            ambient_physics::fetch_simulation_system(),
            Box::new(ambient_physics::physx::sync_ecs_physics()),
            Box::new(ambient_core::transform::TransformSystem::new()),
            ambient_core::remove_at_time_system(),
            Box::new(WorldEventsSystem),
            Box::new(ambient_physics::server_systems()),
            Box::new(shared::player::server_systems()),
            Box::new(wasm::systems()),
            Box::new(shared::player::server_systems_final()),
        ],
    )
}
fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    SystemGroup::new("on_forking_systems", vec![Box::new(ambient_physics::on_forking_systems()), Box::new(wasm::on_forking_systems())])
}
fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    SystemGroup::new("on_shutdown_systems", vec![Box::new(ambient_physics::on_shutdown_systems()), Box::new(wasm::on_shutdown_systems())])
}

fn is_sync_component(component: ComponentDesc, _: WorldStreamCompEvent) -> bool {
    component.has_attribute::<Networked>()
}

fn create_resources(assets: AssetCache) -> Entity {
    let mut server_resources = Entity::new().with(asset_cache(), assets.clone()).with(no_sync(), ()).with_default(world_events());

    ambient_physics::create_server_resources(&assets, &mut server_resources);

    server_resources.merge(ambient_core::async_ecs::async_ecs_resources());
    server_resources.set_self(ambient_core::runtime(), RuntimeHandle::current());
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    server_resources.set_self(time(), now);
    server_resources.set_self(app_start_time(), now);
    server_resources.set_self(dtime(), 1. / 60.);

    let mut handlers = HashMap::new();
    ambient_network::register_rpc_bi_stream_handler(&mut handlers, shared::create_rpc_registry());
    server_resources.set_self(bi_stream_handlers(), handlers);

    let mut handlers = HashMap::new();
    shared::player::register_datagram_handler(&mut handlers);
    server_resources.set_self(datagram_handlers(), handlers);

    server_resources
}

pub const HTTP_INTERFACE_PORT: u16 = 8999;
pub const QUIC_INTERFACE_PORT: u16 = 9000;

fn start_http_interface(runtime: &tokio::runtime::Runtime, project_path: &Path) {
    let router = Router::new()
        .route("/ping", get(|| async move { "ok" }))
        .nest_service("/content", get_service(ServeDir::new(project_path.join("build"))).handle_error(handle_error))
        .layer(CorsLayer::new().allow_origin(tower_http::cors::Any).allow_methods(vec![Method::GET]).allow_headers(tower_http::cors::Any));

    runtime.spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], HTTP_INTERFACE_PORT));
        axum::Server::bind(&addr).serve(router.into_make_service()).await.unwrap();
    });
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
