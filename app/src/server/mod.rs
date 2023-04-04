use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use ambient_core::{app_start_time, asset_cache, dtime, no_sync, project_name, time};
use ambient_ecs::{
    dont_store, world_events, ComponentDesc, ComponentRegistry, Entity, Networked, SystemGroup, World, WorldEventsSystem,
    WorldStreamCompEvent,
};
use ambient_network::{
    persistent_resources,
    server::{ForkingEvent, GameServer, ShutdownEvent, ProxySettings},
    synced_resources,
};
use ambient_prefab::PrefabFromUrl;
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ServerBaseUrlKey},
};
use ambient_sys::{task::RuntimeHandle, time::SystemTime};
use anyhow::Context;
use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{cli::Cli, shared};

pub mod wasm;

pub fn start(
    runtime: &tokio::runtime::Runtime,
    assets: AssetCache,
    cli: Cli,
    project_path: PathBuf,
    manifest: &ambient_project::Manifest,
) -> u16 {
    log::info!("Creating server");
    let host_cli = cli.host().unwrap();
    let quic_interface_port = host_cli.quic_interface_port;
    let proxy_settings = (!host_cli.no_proxy).then(|| {
        ProxySettings {
            // default to getting a proxy from the dims-web Google App Engine app
            endpoint: host_cli.proxy.clone().unwrap_or("http://proxy.ambient.run/proxy".to_string()),
            project_path: project_path.clone(),
            pre_cache_assets: host_cli.proxy_pre_cache_assets,
            project_id: manifest.project.id.to_string(),
        }
    });
    let server = runtime.block_on(async move {
        if let Some(port) = quic_interface_port {
            GameServer::new_with_port(port, false, proxy_settings).await.context("failed to create game server with port").unwrap()
        } else {
            GameServer::new_with_port_in_range(QUIC_INTERFACE_PORT..(QUIC_INTERFACE_PORT + 10), false, proxy_settings)
                .await
                .context("failed to create game server with port in range")
                .unwrap()
        }
    });
    let port = server.port;

    let public_host = cli
        .host()
        .and_then(|h| h.public_host.clone())
        .or_else(|| local_ip_address::local_ip().ok().map(|x| x.to_string()))
        .unwrap_or("localhost".to_string());
    log::info!("Created server, running at {public_host}:{port}");
    let http_interface_port = cli.host().unwrap().http_interface_port.unwrap_or(HTTP_INTERFACE_PORT);
    ServerBaseUrlKey.insert(&assets, AbsAssetUrl::parse(format!("http://{public_host}:{http_interface_port}/content/")).unwrap());

    start_http_interface(runtime, &project_path, http_interface_port);

    ComponentRegistry::get_mut().add_external(ambient_project::all_defined_components(manifest, false).unwrap());

    let manifest = manifest.clone();
    runtime.spawn(async move {
        let mut server_world = World::new_with_config("server", true);
        server_world.init_shape_change_tracking();

        server_world.add_components(server_world.resource_entity(), create_resources(assets.clone())).unwrap();

        // Keep track of the project name
        let name = manifest.project.name.clone().unwrap_or_else(|| "Ambient".into());
        server_world.add_components(server_world.resource_entity(), Entity::new().with(project_name(), name)).unwrap();

        Entity::new().with(synced_resources(), ()).with(dont_store(), ()).spawn(&mut server_world);
        // Note: this should not be reset every time the server is created. Remove this when it becomes possible to load/save worlds.
        Entity::new().with(persistent_resources(), ()).spawn(&mut server_world);

        wasm::initialize(&mut server_world, project_path.clone(), &manifest).unwrap();

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
            Box::new(ambient_core::camera::camera_systems()),
            Box::new(ambient_physics::server_systems()),
            Box::new(wasm::systems()),
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
    server_resources.set(ambient_core::runtime(), RuntimeHandle::current());

    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    server_resources.set(time(), now);
    server_resources.set(app_start_time(), now);
    server_resources.set(dtime(), 1. / 60.);

    let mut bistream_handlers = HashMap::new();
    ambient_network::server::register_rpc_bi_stream_handler(&mut bistream_handlers, shared::create_server_rpc_registry());
    server_resources.set(ambient_network::server::bi_stream_handlers(), bistream_handlers);

    let unistream_handlers = HashMap::new();
    server_resources.set(ambient_network::server::uni_stream_handlers(), unistream_handlers);

    let dgram_handlers = HashMap::new();
    server_resources.set(ambient_network::server::datagram_handlers(), dgram_handlers);

    server_resources
}

pub const HTTP_INTERFACE_PORT: u16 = 8999;
pub const QUIC_INTERFACE_PORT: u16 = 9000;
fn start_http_interface(runtime: &tokio::runtime::Runtime, project_path: &Path, http_interface_port: u16) {
    let router = Router::new()
        .route("/ping", get(|| async move { "ok" }))
        .nest_service("/content", get_service(ServeDir::new(project_path.join("build"))).handle_error(handle_error))
        .layer(CorsLayer::new().allow_origin(tower_http::cors::Any).allow_methods(vec![Method::GET]).allow_headers(tower_http::cors::Any));

    let serve = |addr| async move {
        axum::Server::try_bind(&addr)?.serve(router.into_make_service()).await?;

        Ok::<_, anyhow::Error>(())
    };

    runtime.spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], http_interface_port));

        if let Err(err) = serve(addr).await {
            tracing::error!("Failed to start server on: {addr}\n\n{err:?}");
        }
    });
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
