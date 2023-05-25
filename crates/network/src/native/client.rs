use crate::{
    client::{GameClient, GameClientRenderTarget, LoadedFunc, NetworkStats},
    client_game_state::ClientGameState,
    native::load_root_certs,
    proto::{
        client::{ClientState, SharedClientState},
        ClientRequest,
    },
    server::RpcArgs,
    stream::{self, RecvStream, SendStream},
    NetworkError,
};
use ambient_app::window_title;
use ambient_core::{asset_cache, gpu};
use ambient_ecs::{generated::messages, world_events, Entity, SystemGroup};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_renderer::RenderTarget;
use ambient_rpc::RpcRegistry;
use ambient_std::{cb, Cb};
use ambient_ui_native::{Centered, FlowColumn, FlowRow, Text, Throbber};
use anyhow::Context;
use futures::{SinkExt, StreamExt};
use glam::uvec2;
use parking_lot::Mutex;
use quinn::{ClientConfig, Connection, Endpoint, TransportConfig};
use rand::Rng;
use rustls::Certificate;
use tokio::net::ToSocketAddrs;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct ResolvedAddr {
    pub host_name: String,
    pub addr: SocketAddr,
}

impl ResolvedAddr {
    pub async fn lookup_host<T: ToSocketAddrs + ToString + Clone>(host: T) -> anyhow::Result<Self> {
        let addr = tokio::net::lookup_host(host.clone())
            .await?
            .find(SocketAddr::is_ipv4)
            .ok_or_else(|| anyhow::anyhow!("No IPv4 addresses found for: {}", host.to_string()))?;
        let host = host.to_string();
        let host_name = if host.contains(':') {
            host.split(':').next().unwrap().to_string()
        }else {
            host
        };
        Ok(Self { host_name, addr })
    }

    pub fn localhost_with_port(port: u16) -> Self {
        Self {
            host_name: "localhost".into(),
            addr: ([127, 0, 0, 1], port).into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameClientView {
    pub server_addr: ResolvedAddr,
    pub cert: Option<Vec<u8>>,
    pub user_id: String,
    pub systems_and_resources: Cb<dyn Fn() -> (SystemGroup, Entity) + Sync + Send>,
    pub error_view: Cb<dyn Fn(String) -> Element + Sync + Send>,
    pub on_loaded: LoadedFunc,
    pub create_rpc_registry: Cb<dyn Fn() -> RpcRegistry<RpcArgs> + Sync + Send>,
    pub inner: Element,
}

impl ElementComponent for GameClientView {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self {
            server_addr,
            user_id,
            error_view,
            systems_and_resources,
            create_rpc_registry,
            on_loaded,
            inner,
            cert,
        } = *self;

        let gpu = hooks.world.resource(gpu()).clone();

        hooks.provide_context(|| {
            GameClientRenderTarget(Arc::new(RenderTarget::new(gpu.clone(), uvec2(1, 1), None)))
        });
        let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();

        let assets = hooks.world.resource(asset_cache()).clone();
        let game_state = hooks.use_ref_with(|world| {
            let (systems, resources) = systems_and_resources();

            ClientGameState::new(
                world,
                assets.clone(),
                user_id.clone(),
                render_target.0.clone(),
                systems,
                resources,
            )
        });

        let ((control_tx, control_rx), _) = hooks.use_state_with(|_| flume::unbounded());

        // The game client will be set once a connection establishes
        let (game_client, set_game_client) = hooks.use_state(None as Option<GameClient>);

        // Subscribe to window close events
        hooks.use_runtime_message::<messages::WindowClose>({
            move |_, _| {
                tracing::info!("User closed the window");
                control_tx.send(Control::Disconnect).ok();
            }
        });

        // Run game logic
        {
            let game_state = game_state.clone();
            let render_target = render_target.clone();
            let world_event_reader = Mutex::new(hooks.world.resource(world_events()).reader());

            let game_client_exists = game_client.is_some();
            hooks.use_frame(move |app_world| {
                if !game_client_exists {
                    return;
                }

                let mut game_state = game_state.lock();

                // Pipe events from app world to game world
                for (_, event) in world_event_reader
                    .lock()
                    .iter(app_world.resource(world_events()))
                {
                    game_state
                        .world
                        .resource_mut(world_events())
                        .add_event(event.clone());
                }

                // tracing::info!("Drawing game state");
                game_state.on_frame(&render_target.0);
            });
        }

        // Set the window title to the project name
        let (window_title_state, _set_window_title) = hooks.use_state("Ambient".to_string());
        *hooks.world.resource_mut(window_title()) = window_title_state;

        let (error, set_error) = hooks.use_state(None);

        hooks.use_task(move |_| {
            let task = async move {
                let conn = open_connection(server_addr.clone(), cert.map(Certificate))
                    .await
                    .with_context(|| format!("Failed to connect to endpoint: {server_addr:?}"))?;

                tracing::info!("Connected to the server");

                // Create a handle for the game client
                let game_client = GameClient::new(
                    Arc::new(conn.clone()),
                    Arc::new(create_rpc_registry()),
                    game_state.clone(),
                    user_id.clone(),
                );

                handle_connection(
                    game_client,
                    conn,
                    user_id,
                    ClientCallbacks {
                        on_loaded: cb(move |game_client| {
                            let game_state = &game_client.game_state;
                            {
                                // Updates the game client context in the Ui tree
                                set_game_client(Some(game_client.clone()));
                                // Update the resources on the client side world to reflect the new connection
                                // state
                                let world = &mut game_state.lock().world;
                                world.add_resource(
                                    crate::client::game_client(),
                                    Some(game_client.clone()),
                                );
                            }
                            (on_loaded)(game_client)
                        }),
                    },
                    game_state,
                    control_rx,
                )
                .await?;

                tracing::info!("Finished handling connection");

                Ok(()) as anyhow::Result<()>
            };

            async move {
                match task.await {
                    Ok(()) => {
                        tracing::info!("Client disconnected");
                    }
                    Err(err) => {
                        if let Some(err) = err.downcast_ref::<NetworkError>() {
                            if let NetworkError::ConnectionClosed = err {
                                tracing::info!("Connection closed by peer");
                            } else {
                                tracing::error!("Network error: {:?}", err);
                            }
                        } else {
                            tracing::error!("Game failed: {:?}", err);
                        }
                        set_error(Some(format!("{err:?}")));
                    }
                }
            }
        });

        if let Some(err) = error {
            return error_view(err);
        }

        if let Some(game_client) = game_client {
            // Provide the context
            hooks.provide_context(|| game_client.clone());
            hooks
                .world
                .add_resource(crate::client::game_client(), Some(game_client.clone()));

            inner
        } else {
            Centered(vec![FlowColumn::el([FlowRow::el([
                Text::el("Connecting"),
                Throbber.el(),
            ])])])
            .el()
        }
    }
}

#[derive(Debug)]
struct ClientCallbacks {
    on_loaded: LoadedFunc,
}

pub enum Control {
    Disconnect,
}

#[tracing::instrument(name = "client", level = "info", skip(conn))]
async fn handle_connection(
    game_client: GameClient,
    conn: quinn::Connection,
    user_id: String,
    callbacks: ClientCallbacks,
    state: SharedClientState,
    control_rx: flume::Receiver<Control>,
) -> anyhow::Result<()> {
    tracing::info!("Handling client connection");
    tracing::info!("Opening control stream");

    let mut request_send = SendStream::new(conn.open_uni().await?);

    tracing::info!("Opened control stream");

    // Accept the diff and stat stream
    // Nothing is read from them until the connection has been accepted

    // Send a connection request
    tracing::info!("Attempting to connect using {user_id:?}");

    request_send
        .send(ClientRequest::Connect(user_id.clone()))
        .await?;

    let mut client = ClientState::Connecting(user_id);

    tracing::info!("Accepting control stream from server");
    let mut push_recv = stream::RecvStream::new(conn.accept_uni().await?);

    tracing::info!("Entering client loop");
    while client.is_connecting() {
        tracing::info!("Waiting for server to accept connection and send server info");
        if let Some(frame) = push_recv.next().await {
            client.process_push(&state, frame?)?;
        }
    }

    tracing::info!("Accepting diff stream");
    let mut diff_stream = RecvStream::new(conn.accept_uni().await?);

    let cleanup = (callbacks.on_loaded)(game_client)?;
    let on_disconnect = move || {
        tracing::info!("Running connection cleanup");
        cleanup()
    };

    scopeguard::defer!(on_disconnect());

    let stats_interval = 5;
    let mut stats_timer = tokio::time::interval(Duration::from_secs_f32(stats_interval as f32));
    let mut prev_stats = conn.stats();

    let mut control_rx = control_rx.into_stream();

    tracing::info!("Client connected");

    while let ClientState::Connected(connected) = &mut client {
        tokio::select! {
            Some(frame) = push_recv.next() => {
                client.process_push(&state, frame?)?;
            }
            _ = stats_timer.tick() => {
                let stats = conn.stats();

                client.process_client_stats(&state, NetworkStats {
                    latency_ms: conn.rtt().as_millis() as u64,
                    bytes_sent: (stats.udp_tx.bytes - prev_stats.udp_tx.bytes) / stats_interval,
                    bytes_received: (stats.udp_rx.bytes - prev_stats.udp_rx.bytes) / stats_interval,
                });

                prev_stats = stats;
            }

           Some(control) = control_rx.next() => {
                match control {
                    Control::Disconnect => {
                        tracing::info!("Disconnecting manually");
                        // Tell the server that we want to gracefully disconnect
                        request_send.send(ClientRequest::Disconnect).await?;
                    }
                }
            }

            Ok(datagram) = conn.read_datagram() => {
                connected.process_datagram(&state, datagram)?;
            }
            Ok((send, recv)) = conn.accept_bi() => {
                connected.process_bi(&state, send, recv).await?;
            }
            Ok(recv) = conn.accept_uni() => {
                connected.process_uni(&state, recv).await?;
            }
            Some(diff) = diff_stream.next() => {
                connected.process_diff(&state, diff?)?;
            }
        }
    }

    tracing::info!("Client entered disconnected state");
    Ok(())
}

/// Connnect to the server endpoint.
#[tracing::instrument(level = "debug")]
async fn open_connection(
    server_addr: ResolvedAddr,
    cert: Option<Certificate>,
) -> anyhow::Result<Connection> {
    log::debug!("Connecting to world instance: {server_addr:?}");

    let endpoint =
        create_client_endpoint_random_port(cert).context("Failed to create client endpoint")?;

    log::debug!("Got endpoint");
    let conn = endpoint.connect(server_addr.addr, &server_addr.host_name)?.await?;

    log::debug!("Got connection");
    Ok(conn)
}

pub fn create_client_endpoint_random_port(cert: Option<Certificate>) -> anyhow::Result<Endpoint> {
    let mut roots = load_root_certs();

    if let Some(cert) = cert {
        roots
            .add(&cert)
            .context("Failed to add custom certificate")?;
    }

    for _ in 0..10 {
        let client_port = {
            let mut rng = rand::thread_rng();
            rng.gen_range(15000..25000)
        };

        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), client_port);

        if let Ok(mut endpoint) = Endpoint::client(client_addr) {
            let mut tls_config = rustls::ClientConfig::builder()
                .with_safe_default_cipher_suites()
                .with_safe_default_kx_groups()
                .with_protocol_versions(&[&rustls::version::TLS13])
                .unwrap()
                .with_root_certificates(roots)
                .with_no_client_auth();

            // tls_config.enable_early_data = true;
            tls_config.alpn_protocols = vec!["ambient-02".into()];

            let mut transport = TransportConfig::default();
            transport.keep_alive_interval(Some(Duration::from_secs_f32(1.)));

            if std::env::var("AMBIENT_DISABLE_TIMEOUT").is_ok() {
                transport.max_idle_timeout(None);
            } else {
                transport.max_idle_timeout(Some(Duration::from_secs_f32(60.).try_into().unwrap()));
            }
            let mut client_config = ClientConfig::new(Arc::new(tls_config));
            client_config.transport_config(Arc::new(transport));

            endpoint.set_default_client_config(client_config);
            return Ok(endpoint);
        }
    }

    Err(anyhow::anyhow!(
        "Failed to find appropriate port for client endpoint"
    ))
}
