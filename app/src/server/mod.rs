use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use ambient_core::{asset_cache, name, no_sync, project_name, FIXED_SERVER_TICK_TIME};
use ambient_ecs::{
    dont_store, world_events, ComponentDesc, Entity, Networked, SystemGroup, World,
    WorldEventsSystem, WorldStreamCompEvent,
};
use ambient_network::{
    native::server::{Crypto, GameServer},
    persistent_resources,
    server::{ForkingEvent, ProxySettings, ShutdownEvent},
    synced_resources,
};
use ambient_prefab::PrefabFromUrl;
use ambient_project_semantic::{ItemId, Scope, Semantic};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ContentBaseUrlKey, ServerBaseUrlKey},
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

use crate::{
    cli::{Cli, Commands, HostCli},
    shared,
};

pub mod wasm;

#[allow(clippy::too_many_arguments)]
pub async fn start(
    runtime: &tokio::runtime::Handle,
    assets: AssetCache,
    cli: Cli,
    working_directory: PathBuf,
    build_path: AbsAssetUrl,
    semantic: &Semantic,
    primary_ember_id: ItemId<Scope>,
    crypto: Crypto,
) -> SocketAddr {
    let host_cli = cli.host().unwrap();
    let quic_interface_port = host_cli.quic_interface_port;
    let manifest = semantic
        .items
        .get(primary_ember_id)
        .unwrap()
        .manifest
        .clone()
        .expect("primary ember must have manifest");

    let proxy_settings = (!host_cli.no_proxy).then(|| ProxySettings {
        // default to getting a proxy from the dims-web Google App Engine app
        endpoint: host_cli
            .proxy
            .clone()
            .unwrap_or("http://proxy.ambient.run/proxy".to_string()),
        build_path: build_path.clone(),
        pre_cache_assets: host_cli.proxy_pre_cache_assets,
        primary_ember_id: manifest.ember.id.to_string(),
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
    let http_interface_port = cli
        .host()
        .unwrap()
        .http_interface_port
        .unwrap_or(HTTP_INTERFACE_PORT);

    let public_host = match (cli.host().as_ref(), addr.ip()) {
        // use public_host if specified in cli
        (
            Some(&HostCli {
                public_host: Some(host),
                ..
            }),
            _,
        ) => host.clone(),

        // if the bind address is not specified (0.0.0.0, ::0) then use localhost
        (_, IpAddr::V4(Ipv4Addr::UNSPECIFIED)) => IpAddr::V4(Ipv4Addr::LOCALHOST).to_string(),
        (_, IpAddr::V6(Ipv6Addr::UNSPECIFIED)) => IpAddr::V6(Ipv6Addr::LOCALHOST).to_string(),

        // otherwise use the address that the server is binding to
        (_, addr) => addr.to_string(),
    };

    // here the key is inserted into the asset cache
    if let Ok(Some(build_path_fs)) = build_path.to_file_path() {
        let key = format!("http://{public_host}:{http_interface_port}/content/");
        let base_url = AbsAssetUrl::from_str(&key).unwrap();
        ServerBaseUrlKey.insert(&assets, base_url.clone());
        ContentBaseUrlKey.insert(&assets, base_url);

        start_http_interface(runtime, Some(&build_path_fs), http_interface_port);
    } else {
        let base_url = build_path.clone();

        ServerBaseUrlKey.insert(&assets, base_url.clone());
        ContentBaseUrlKey.insert(&assets, base_url);

        start_http_interface(runtime, None, http_interface_port);
    }

    runtime.spawn(async move {
        let mut server_world = World::new_with_config("server", true);
        server_world.init_shape_change_tracking();

        server_world
            .add_components(
                server_world.resource_entity(),
                create_resources(assets.clone()),
            )
            .unwrap();

        // Keep track of the project name
        let name = manifest
            .ember
            .name
            .clone()
            .unwrap_or_else(|| "Ambient".into());
        server_world
            .add_components(
                server_world.resource_entity(),
                Entity::new().with(project_name(), name),
            )
            .unwrap();

        Entity::new()
            .with(ambient_core::name(), "Synced resources".to_string())
            .with(synced_resources(), ())
            .with(dont_store(), ())
            .spawn(&mut server_world);
        // Note: this should not be reset every time the server is created. Remove this when it becomes possible to load/save worlds.
        Entity::new()
            .with(ambient_core::name(), "Persistent resources".to_string())
            .with(persistent_resources(), ())
            .spawn(&mut server_world);

        wasm::initialize(
            &mut server_world,
            working_directory.join("data"),
            // TODO: do not support rebuilds with ember yet
            None,
            // build_config.map(|config| {
            //     // HACK: provide a callback to rebuild the project to WASM.
            //     // this is not done directly within WASM due to circular dependencies.
            //     cb(move |world: &mut World| {
            //         let runtime = world.resource(ambient_core::runtime()).clone();
            //         let async_run = world.resource(ambient_core::async_ecs::async_run()).clone();
            //         let (path, manifest, build_path, optimize) = (
            //             config.path.clone(),
            //             config.manifest.clone(),
            //             config.build_path(),
            //             config.optimize,
            //         );
            //         runtime.spawn(async move {
            //             let result = ambient_build::build_rust_if_available(
            //                 &path,
            //                 &manifest,
            //                 &build_path,
            //                 optimize,
            //             )
            //             .await;

            //             async_run.run(|world| {
            //                 world.resource_mut(ambient_ecs::world_events()).add_message(
            //                     messages::WasmRebuild::new(result.err().map(|err| err.to_string())),
            //                 );
            //             });
            //         });
            //     }) as Cb<dyn Fn(&mut World) + Send + Sync>
            // }),
        )
        .await
        .unwrap();

        if let Commands::View { asset_path, .. } = cli.command.clone() {
            let asset_path = build_path
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
        .with_default(world_events());
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
fn start_http_interface(
    runtime: &tokio::runtime::Handle,
    build_path: Option<&Path>,
    http_interface_port: u16,
) {
    let mut router = Router::new().route("/ping", get(|| async move { "ok" }));

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

    runtime.spawn(async move {
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
