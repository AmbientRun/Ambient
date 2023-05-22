use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Range,
    sync::Arc,
    time::Duration,
};

use ambient_core::{asset_cache, no_sync, project_name};
use ambient_ecs::{
    ArchetypeFilter, ComponentDesc, ComponentRegistry, System, SystemGroup, World, WorldStream,
    WorldStreamCompEvent, WorldStreamFilter,
};
use ambient_proxy::client::AllocatedEndpoint;
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ServerBaseUrlKey},
    fps_counter::FpsCounter,
    log_result,
};
use ambient_sys::time::Instant;
use colored::Colorize;
use futures::StreamExt;
use parking_lot::{Mutex, RwLock};
use quinn::{ClientConfig, Endpoint, ServerConfig, TransportConfig};
use rustls::{Certificate, PrivateKey, RootCertStore};
use tokio::time::{interval, MissedTickBehavior};
use uuid::Uuid;

use crate::{
    client_connection::ConnectionKind,
    proto::{
        self,
        server::{handle_diffs, ConnectionData},
        ServerInfo, ServerPush, VERSION,
    },
    server::{
        server_stats, ForkingEvent, ProxySettings, ServerState, SharedServerState, ShutdownEvent,
        WorldInstance, MAIN_INSTANCE_ID,
    },
    stream, ServerWorldExt,
};

#[derive(Debug, Clone)]
pub struct Crypto {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
}

/// Quinn and Webtransport game server
pub struct GameServer {
    endpoint: Endpoint,
    pub port: u16,
    /// Shuts down the server if there are no players
    pub use_inactivity_shutdown: bool,
    proxy_settings: Option<ProxySettings>,
}
impl GameServer {
    pub async fn new_with_port(
        port: u16,
        use_inactivity_shutdown: bool,
        proxy_settings: Option<ProxySettings>,
        crypto: &Crypto,
    ) -> anyhow::Result<Self> {
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

        let endpoint = create_server(server_addr, crypto)?;

        tracing::debug!("GameServer listening on port {}", port);
        Ok(Self {
            endpoint,
            port,
            use_inactivity_shutdown,
            proxy_settings,
        })
    }
    pub async fn new_with_port_in_range(
        port_range: Range<u16>,
        use_inactivity_shutdown: bool,
        proxy_settings: Option<ProxySettings>,
        crypto: &Crypto,
    ) -> anyhow::Result<Self> {
        for port in port_range {
            match Self::new_with_port(
                port,
                use_inactivity_shutdown,
                proxy_settings.clone(),
                crypto,
            )
            .await
            {
                Ok(server) => {
                    return Ok(server);
                }
                Err(err) => {
                    tracing::warn!("Failed to create server on port {port}.\n{err:?}");
                }
            }
        }
        anyhow::bail!("Failed to create server")
    }
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn run(
        self,
        mut world: World,
        create_server_systems: Arc<dyn Fn(&mut World) -> SystemGroup + Sync + Send>,
        create_on_forking_systems: Arc<dyn Fn() -> SystemGroup<ForkingEvent> + Sync + Send>,
        create_shutdown_systems: Arc<dyn Fn() -> SystemGroup<ShutdownEvent> + Sync + Send>,
        is_sync_component: Arc<dyn Fn(ComponentDesc, WorldStreamCompEvent) -> bool + Sync + Send>,
    ) -> SharedServerState {
        let Self {
            endpoint,
            proxy_settings,
            ..
        } = self;
        let assets = world.resource(asset_cache()).clone();
        let world_stream_filter =
            WorldStreamFilter::new(ArchetypeFilter::new().excl(no_sync()), is_sync_component);
        let state = Arc::new(Mutex::new(ServerState::new(
            assets.clone(),
            [(
                MAIN_INSTANCE_ID.to_string(),
                WorldInstance {
                    systems: create_server_systems(&mut world),
                    world,
                    world_stream: WorldStream::new(world_stream_filter.clone()),
                },
            )]
            .into_iter()
            .collect(),
            create_server_systems,
            create_on_forking_systems,
            create_shutdown_systems,
        )));

        let mut fps_counter = FpsCounter::new();
        let mut sim_interval = interval(Duration::from_secs_f32(1. / 60.));
        sim_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut inactivity_interval = interval(Duration::from_secs_f32(5.));
        let mut last_active = ambient_sys::time::Instant::now();

        if let Some(proxy_settings) = proxy_settings {
            let endpoint = endpoint.clone();
            let state = state.clone();
            let world_stream_filter = world_stream_filter.clone();
            let assets = assets.clone();
            tokio::spawn(async move {
                start_proxy_connection(
                    endpoint.clone(),
                    proxy_settings,
                    state.clone(),
                    world_stream_filter.clone(),
                    assets.clone(),
                )
                .await;
            });
        }

        loop {
            tracing::trace_span!("Listening for incoming connections");
            tokio::select! {
                Some(conn) = endpoint.accept() => {
                    tracing::debug!("Received connection");

                    let conn = match conn.await {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!("Failed to accept incoming connection. {e}");
                            continue;
                        }
                    };


                    tracing::debug!("Accepted connection");
                    let fut = handle_quinn_connection(conn.into(), state.clone(), world_stream_filter.clone(), ServerBaseUrlKey.get(&assets));
                    tokio::spawn(async move {  log_result!(fut.await) });
                }
                _ = sim_interval.tick() => {
                    fps_counter.frame_start();
                    let mut state = state.lock();
                    tokio::task::block_in_place(|| {
                        ambient_profiling::finish_frame!();
                        ambient_profiling::scope!("sim_tick");
                        state.step();
                        state.broadcast_diffs();
                        if let Some(sample) = fps_counter.frame_end() {
                            for instance in state.instances.values_mut() {
                                let id = instance.world.synced_resource_entity().unwrap();
                                instance.world.add_component(id, server_stats(), sample.clone()).unwrap();
                            }
                        }
                    });
                }
                _ = inactivity_interval.tick(), if self.use_inactivity_shutdown => {
                    if state.lock().player_count() == 0 {
                        if Instant::now().duration_since(last_active).as_secs_f32() > 2. * 60. {
                            tracing::info!("[{}] Shutting down due to inactivity", self.port);
                            break;
                        }
                    } else {
                        last_active = Instant::now();
                    }
                }
                else => {
                    tracing::info!("No more connections. Shutting down.");
                    break
                }
            }
        }
        tracing::debug!("[{}] GameServer shutting down", self.port);
        {
            let mut state = state.lock();
            let create_shutdown_systems = state.create_shutdown_systems.clone();
            for instance in state.instances.values_mut() {
                let mut sys = (create_shutdown_systems)();
                sys.run(&mut instance.world, &ShutdownEvent);
            }
        }
        tracing::debug!("[{}] GameServer finished shutting down", self.port);
        state
    }
}

/// Setup the protocol and enter the update loop for a new connected client
#[tracing::instrument(name = "server", level = "info", skip_all, fields(content_base_url))]
async fn handle_quinn_connection(
    conn: ConnectionKind,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    content_base_url: AbsAssetUrl,
) -> anyhow::Result<()> {
    tracing::info!("Handling server connection");
    let (diffs_tx, diffs_rx) = flume::unbounded();

    let server_info = {
        let state = state.lock();
        let instance = state.instances.get(MAIN_INSTANCE_ID).unwrap();
        let world = &instance.world;
        let external_components = ComponentRegistry::get()
            .all_external()
            .map(|x| x.0)
            .collect();

        ServerInfo {
            project_name: world.resource(project_name()).clone(),
            content_base_url,
            version: VERSION.into(),
            external_components,
        }
    };

    let mut server = proto::server::ServerState::default();

    tracing::info!("Accepting request stream from client");
    let mut request_recv = stream::RecvStream::new(conn.accept_uni().await?);
    tracing::info!("Opening control stream");
    let mut push_send = stream::SendStream::new(conn.open_uni().await?);

    let diffs_rx = diffs_rx.into_stream();

    use futures::SinkExt;

    // Send who we are
    push_send.send(ServerPush::ServerInfo(server_info)).await?;

    // Feed the channel senders to the connection data
    //
    // Once connected they will be added to the player entity
    let data = ConnectionData {
        conn: Arc::new(conn.clone()),
        state,
        diff_tx: diffs_tx,
        connection_id: Uuid::new_v4(),
        world_stream_filter,
    };

    while server.is_pending_connection() {
        tracing::info!("Waiting for connect request");
        if let Some(frame) = request_recv.next().await {
            server.process_control(&data, frame?)?;
        }
    }

    tracing::debug!("Performing additional on connect tracingic after the fact");

    tokio::spawn(handle_diffs(
        stream::SendStream::new(conn.open_uni().await?),
        diffs_rx,
    ));

    // Before a connection has been established, only process the control stream
    while let proto::server::ServerState::Connected(connected) = &mut server {
        tokio::select! {
            Some(frame) = request_recv.next() => {
                server.process_control(&data, frame?)?;
            }
            stream = conn.accept_uni() => {
                connected.process_uni(&data, stream?).await?;
            }
            stream = conn.accept_bi() => {
                let (send, recv) = stream?;
                connected.process_bi(&data, send, recv).await?;
            }
            datagram = conn.read_datagram() => {
                connected.process_datagram(&data, datagram?).await?;
            }
            Some(msg) = connected.control_rx.next() => {
                push_send.send(&msg).await?;
            }
        }
    }

    tracing::info!("Client disconnected");

    Ok(())
}

async fn start_proxy_connection(
    endpoint: Endpoint,
    settings: ProxySettings,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    assets: AssetCache,
) {
    // start with content base url being the same as for direct connections
    let content_base_url = Arc::new(RwLock::new(ServerBaseUrlKey.get(&assets)));

    let on_endpoint_allocated = {
        let content_base_url = content_base_url.clone();
        Arc::new(
            move |AllocatedEndpoint {
                      id,
                      allocated_endpoint,
                      external_endpoint,
                      assets_root,
                      ..
                  }: AllocatedEndpoint| {
                tracing::debug!("Allocated proxy endpoint. Allocation id: {}", id);
                tracing::info!("Proxy sees this server as {}", external_endpoint);
                tracing::info!(
                    "Proxy allocated an endpoint, use `{}` to join",
                    format!("ambient join {}", allocated_endpoint).bright_green()
                );

                // set the content base url to point to proxy provided value
                match AbsAssetUrl::parse(&assets_root) {
                    Ok(url) => {
                        tracing::debug!("Got content base root from proxy: {}", url);
                        *content_base_url.write() = url;
                    }
                    Err(err) => {
                        tracing::warn!("Failed to parse assets root url ({}): {}", assets_root, err)
                    }
                }
            },
        )
    };

    let on_player_connected = {
        let content_base_url = content_base_url.clone();
        Arc::new(
            move |_player_id, conn: ambient_proxy::client::ProxiedConnection| {
                tracing::debug!("Accepted connection via proxy");
                let task = handle_quinn_connection(
                    conn.into(),
                    state.clone(),
                    world_stream_filter.clone(),
                    content_base_url.read().clone(),
                );

                tokio::spawn(async move { log_result!(task.await) });
            },
        )
    };

    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

    let builder = ambient_proxy::client::builder()
        .endpoint(endpoint.clone())
        .proxy_server(settings.endpoint.clone())
        .project_id(settings.project_id.clone())
        .user_agent(APP_USER_AGENT.to_string());

    let assets_path = settings
        .project_path
        .push("build")
        .expect("Pushing to path cannot fail");
    let builder = if let Ok(Some(assets_file_path)) = assets_path.to_file_path() {
        builder.assets_path(assets_file_path)
    } else {
        builder.assets_root_override(content_base_url.read().to_string())
    };

    tracing::info!("Connecting to proxy server");
    let proxy = match builder.build().await {
        Ok(proxy_client) => proxy_client,
        Err(err) => {
            tracing::warn!("Failed to connect to proxy: {}", err);
            return;
        }
    };

    // start and allocate endpoint
    let mut controller = proxy.start(on_endpoint_allocated, on_player_connected);
    tracing::info!("Allocating proxy endpoint");
    if let Err(err) = controller.allocate_endpoint().await {
        tracing::warn!("Failed to allocate proxy endpoint: {}", err);
    }

    // pre-cache "assets" subdirectory
    if settings.pre_cache_assets {
        for subdir in ["assets", "client"] {
            if let Err(err) = controller.pre_cache_assets(subdir) {
                tracing::warn!("Failed to pre-cache assets: {}", err);
            }
        }
    }
}

fn create_server(server_addr: SocketAddr, crypto: &Crypto) -> anyhow::Result<Endpoint> {
    let mut tls_config = rustls::ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![Certificate(crypto.cert.clone())],
            PrivateKey(crypto.key.clone()),
        )?;

    tls_config.max_early_data_size = u32::MAX;
    let alpn: Vec<Vec<u8>> = vec![
        b"h3".to_vec(),
        b"h3-32".to_vec(),
        b"h3-31".to_vec(),
        b"h3-30".to_vec(),
        b"h3-29".to_vec(),
        b"ambient-02".to_vec(),
    ];

    tls_config.alpn_protocols = alpn;

    let mut server_conf = ServerConfig::with_crypto(Arc::new(tls_config));
    let mut transport = TransportConfig::default();

    transport.keep_alive_interval(Some(Duration::from_secs(2)));

    if std::env::var("AMBIENT_DISABLE_TIMEOUT").is_ok() {
        transport.max_idle_timeout(None);
    } else {
        transport.max_idle_timeout(Some(Duration::from_secs_f32(60.).try_into()?));
    }

    let transport = Arc::new(transport);
    server_conf.transport = transport.clone();

    tracing::info!(?server_addr, ?server_conf, "Creating server endpoint");

    let mut endpoint = Endpoint::server(server_conf, server_addr)?;

    // Create client config for the server endpoint for proxying and hole punching
    let mut roots = RootCertStore::empty();
    roots.add(&Certificate(crypto.cert.clone())).unwrap();
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let mut client_config = ClientConfig::new(Arc::new(crypto));
    client_config.transport_config(transport);
    endpoint.set_default_client_config(client_config);

    Ok(endpoint)
}
