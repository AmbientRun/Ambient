use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};

use ambient_core::{asset_cache, name, no_sync, package_name, FIXED_SERVER_TICK_TIME};
use ambient_ecs::{
    dont_store, world_events, ComponentDesc, Entity, Networked, SystemGroup, World,
    WorldEventsSystem, WorldStreamCompEvent,
};
use ambient_native_std::{
    ambient_version,
    asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ContentBaseUrlKey, ServerBaseUrlKey},
};
use ambient_network::{
    is_persistent_resources, is_synced_resources,
    native::server::{Crypto, GameServer},
    server::{ForkingEvent, ProxySettings, ShutdownEvent},
};
use ambient_prefab::PrefabFromUrl;
use ambient_sys::task::RuntimeHandle;
use anyhow::Context;
use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{cli::HostCli, shared};

pub mod wasm;

#[allow(clippy::too_many_arguments)]
pub async fn start(
    assets: AssetCache,
    host_cli: &HostCli,
    build_root_path: AbsAssetUrl,
    main_package_path: AbsAssetUrl,
    view_asset_path: Option<PathBuf>,
    working_directory: PathBuf,
    manifest: ambient_package::Manifest,
    crypto: Crypto,
) -> SocketAddr {
    let quic_interface_port = host_cli.quic_interface_port;

    let proxy_settings = (!host_cli.no_proxy).then(|| ProxySettings {
        // default to getting a proxy from the dims-web Google App Engine app
        endpoint: host_cli
            .proxy
            .clone()
            .unwrap_or("http://proxy.ambient.run/proxy".to_string()),
        build_path: build_root_path.clone(),
        pre_cache_assets: host_cli.proxy_pre_cache_assets,
        primary_package_id: manifest.package.id.to_string(),
    });

    let server = if let Some(port) = quic_interface_port {
        GameServer::new_with_port(
            SocketAddr::new(host_cli.bind_address, port),
            false,
            proxy_settings,
            &crypto,
        )
        .await
        .context("failed to create game server with port")
        .unwrap()
    } else {
        GameServer::new_with_port_in_range(
            host_cli.bind_address,
            QUIC_INTERFACE_PORT..(QUIC_INTERFACE_PORT + 10),
            false,
            proxy_settings,
            &crypto,
        )
        .await
        .context("failed to create game server with port in range")
        .unwrap()
    };

    let addr = server.local_addr();

    tracing::info!("Created server, running at {addr}");
    let http_interface_port = host_cli.http_interface_port.unwrap_or(HTTP_INTERFACE_PORT);

    let public_host = match (&host_cli.public_host, addr.ip()) {
        // use public_host if specified in cli
        (Some(host), _) => host.clone(),

        // if the bind address is not specified (0.0.0.0, ::0) then use localhost
        (_, IpAddr::V4(Ipv4Addr::UNSPECIFIED)) => IpAddr::V4(Ipv4Addr::LOCALHOST).to_string(),
        (_, IpAddr::V6(Ipv6Addr::UNSPECIFIED)) => IpAddr::V6(Ipv6Addr::LOCALHOST).to_string(),

        // otherwise use the address that the server is binding to
        (_, addr) => addr.to_string(),
    };

    // here the key is inserted into the asset cache
    if let Ok(Some(build_path_fs)) = build_root_path.to_file_path() {
        let key = format!("http://{public_host}:{http_interface_port}/content/");
        let base_url = AbsAssetUrl::from_str(&key).unwrap();
        ServerBaseUrlKey.insert(&assets, base_url.clone());
        ContentBaseUrlKey.insert(&assets, base_url);

        start_http_interface(Some(&build_path_fs), http_interface_port);
    } else {
        let base_url = build_root_path.clone();

        ServerBaseUrlKey.insert(&assets, base_url.clone());
        ContentBaseUrlKey.insert(&assets, base_url);

        start_http_interface(None, http_interface_port);
    }

    tokio::task::spawn(async move {
        let mut server_world = World::new_with_config("server", true);
        server_world.init_shape_change_tracking();

        server_world
            .add_components(
                server_world.resource_entity(),
                create_resources(assets.clone()),
            )
            .unwrap();

        // Keep track of the package name
        let name = manifest
            .package
            .name
            .clone()
            .unwrap_or_else(|| "Ambient".into());
        server_world
            .add_components(
                server_world.resource_entity(),
                Entity::new().with(package_name(), name),
            )
            .unwrap();

        Entity::new()
            .with(ambient_core::name(), "Synced resources".to_string())
            .with(is_synced_resources(), ())
            .with(dont_store(), ())
            .with(
                ambient_package_semantic_native::package_name_to_url(),
                Default::default(),
            )
            .spawn(&mut server_world);
        // Note: this should not be reset every time the server is created. Remove this when it becomes possible to load/save worlds.
        Entity::new()
            .with(ambient_core::name(), "Persistent resources".to_string())
            .with(is_persistent_resources(), ())
            .spawn(&mut server_world);

        wasm::initialize(&mut server_world, working_directory.join("data"))
            .await
            .unwrap();

        let mut semantic = ambient_package_semantic::Semantic::new().await.unwrap();
        let primary_package_scope_id = shared::package::add(
            Some(&mut server_world),
            &mut semantic,
            &main_package_path.push("ambient.toml").unwrap(),
        )
        .await
        .unwrap();

        let mut queue = semantic
            .items
            .scope_and_dependencies(primary_package_scope_id);
        queue.reverse();

        // Use the topologically sorted queue to construct a dict of which packages should be on by default.
        // Assume all are on by default, and then update their state based on what packages "closer to the root"
        // state. The last element should be the root.
        let mut package_id_to_enabled = queue
            .iter()
            .map(|&id| (id, true))
            .collect::<HashMap<_, _>>();
        for &package_id in &queue {
            let package = semantic.items.get(package_id);

            for dependency in package.dependencies.values() {
                package_id_to_enabled.insert(dependency.id, dependency.enabled);
            }
        }

        server_world
            .add_component(
                server_world.resource_entity(),
                ambient_package_semantic_native::semantic(),
                Arc::new(Mutex::new(semantic)),
            )
            .unwrap();

        while let Some(package_id) = queue.pop() {
            wasm::instantiate_package(
                &mut server_world,
                package_id,
                package_id_to_enabled
                    .get(&package_id)
                    .copied()
                    .unwrap_or(true),
            )
            .unwrap();
        }

        if let Some(asset_path) = view_asset_path {
            let asset_path = main_package_path
                .push(asset_path.to_string_lossy())
                .expect("FIXME")
                .push("prefabs/main.json")
                .expect("pushing 'prefabs/main.json' shouldn't fail");
            log::info!("Spawning asset from {:?}", asset_path);
            let obj = PrefabFromUrl(asset_path.into()).get(&assets).await.unwrap();
            obj.spawn_into_world(&mut server_world, None);
        }
        log::info!("Starting server");
        server
            .run(
                server_world,
                Arc::new(systems),
                Arc::new(on_forking_systems),
                Arc::new(on_shutdown_systems),
                Arc::new(is_sync_component),
            )
            .await;
    });

    addr
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
            Box::new(ambient_animation::animation_systems()),
            Box::new(ambient_physics::physx::sync_ecs_physics()),
            Box::new(ambient_core::transform::TransformSystem::new()),
            ambient_core::remove_at_time_system(),
            ambient_core::refcount_system(),
            Box::new(WorldEventsSystem),
            Box::new(ambient_core::camera::camera_systems()),
            Box::new(ambient_physics::server_systems()),
            Box::new(wasm::systems()),
        ],
    )
}
fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    SystemGroup::new(
        "on_forking_systems",
        vec![
            Box::new(ambient_physics::on_forking_systems()),
            Box::new(wasm::on_forking_systems()),
        ],
    )
}
fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    SystemGroup::new(
        "on_shutdown_systems",
        vec![
            Box::new(ambient_physics::on_shutdown_systems()),
            Box::new(wasm::on_shutdown_systems()),
        ],
    )
}

fn is_sync_component(component: ComponentDesc, _: WorldStreamCompEvent) -> bool {
    component.has_attribute::<Networked>()
}

fn create_resources(assets: AssetCache) -> Entity {
    let mut server_resources = Entity::new()
        .with(name(), "Resources".to_string())
        .with(asset_cache(), assets.clone())
        .with(no_sync(), ())
        .with(world_events(), Default::default());
    ambient_physics::create_server_resources(&assets, &mut server_resources);
    server_resources.merge(ambient_core::async_ecs::async_ecs_resources());
    server_resources.set(ambient_core::runtime(), RuntimeHandle::current());

    server_resources.merge(ambient_core::time_resources_start(FIXED_SERVER_TICK_TIME));

    let mut bistream_handlers = HashMap::new();
    ambient_network::server::register_rpc_bi_stream_handler(
        &mut bistream_handlers,
        shared::create_server_rpc_registry(),
    );
    server_resources.set(
        ambient_network::server::bi_stream_handlers(),
        bistream_handlers,
    );

    let unistream_handlers = HashMap::new();
    server_resources.set(
        ambient_network::server::uni_stream_handlers(),
        unistream_handlers,
    );

    let dgram_handlers = HashMap::new();
    server_resources.set(ambient_network::server::datagram_handlers(), dgram_handlers);

    server_resources
}

pub const HTTP_INTERFACE_PORT: u16 = 8999;
pub const QUIC_INTERFACE_PORT: u16 = 9000;
fn start_http_interface(build_path: Option<&Path>, http_interface_port: u16) {
    let mut router = Router::new()
        .route("/ping", get(|| async move { "ok" }))
        .route(
            "/info",
            get(|| async move { axum::Json(ambient_version()) }),
        );

    if let Some(build_path) = build_path {
        router = router.nest_service(
            "/content",
            get_service(ServeDir::new(build_path)).handle_error(handle_error),
        );
    };

    router = router.layer(
        CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(vec![Method::GET])
            .allow_headers(tower_http::cors::Any),
    );

    let serve = |addr| async move {
        axum::Server::try_bind(&addr)?
            .serve(router.into_make_service())
            .await?;

        Ok::<_, anyhow::Error>(())
    };

    let build_path = build_path.map(ToOwned::to_owned);

    tokio::task::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], http_interface_port));

        tracing::debug!(?build_path, "Starting HTTP interface on: {addr}");

        if let Err(err) = serve(addr)
            .await
            .with_context(|| format!("Failed to start server on: {addr}"))
        {
            tracing::error!("{err:?}");
        }
    });
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
