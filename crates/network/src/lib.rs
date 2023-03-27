use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use ambient_ecs::{
    components, query, Component, ComponentValue, Debuggable, Description, EntityId, Name, Networked, Serializable, Store, World,
};
use ambient_rpc::{RpcError, RpcRegistry};
use ambient_std::log_error;
use bytes::Bytes;
use futures::{Future, SinkExt, StreamExt};
use quinn::{ClientConfig, Connection, ConnectionClose, ConnectionError::ConnectionClosed, Endpoint, ServerConfig, TransportConfig};
use rand::Rng;
use rustls::{Certificate, PrivateKey, RootCertStore};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub type AsyncMutex<T> = tokio::sync::Mutex<T>;
pub mod client;
pub mod client_connection;
pub mod client_game_state;
pub mod hooks;
pub mod protocol;
pub mod rpc;
pub mod server;

pub const RPC_BISTREAM_ID: u32 = 1;
pub const WASM_BISTREAM_ID: u32 = 2;

pub const WASM_UNISTREAM_ID: u32 = 1;

pub const PLAYER_INPUT_DATAGRAM_ID: u32 = 5;
pub const WASM_DATAGRAM_ID: u32 = 6;

components!("network", {
    /// Works like `world.resource_entity` for server worlds, except it's also persisted to disk, and synchronized to clients
    @[
        Debuggable, Networked,
        Name["Persistent resources"],
        Description["If attached, this entity contains global resources that are persisted to disk and synchronized to clients."]
    ]
    persistent_resources: (),
    /// Works like `world.resource_entity` for server worlds, except it's synchronized to clients. State is not persisted to disk.
    @[
        Debuggable, Networked,
        Name["Synchronized resources"],
        Description["If attached, this entity contains global resources that are synchronized to clients, but not persisted."]
    ]
    synced_resources: (),

    @[
        Debuggable, Networked,
        Name["Is remote entity"],
        Description["If attached, this entity was not spawned locally (e.g. if this is the client, it was spawned by the server)."]
    ]
    is_remote_entity: (),
});

pub fn init_all_components() {
    init_components();
    client::init_components();
    server::init_components();
    client_game_state::init_components();
}

pub trait ServerWorldExt {
    fn persisted_resource_entity(&self) -> Option<EntityId>;
    fn persisted_resource<T: ComponentValue>(&self, component: Component<T>) -> Option<&T>;
    fn persisted_resource_mut<T: ComponentValue>(&mut self, component: Component<T>) -> Option<&mut T>;
    fn synced_resource_entity(&self) -> Option<EntityId>;
    fn synced_resource<T: ComponentValue>(&self, component: Component<T>) -> Option<&T>;
    fn synced_resource_mut<T: ComponentValue>(&mut self, component: Component<T>) -> Option<&mut T>;
}
impl ServerWorldExt for World {
    fn persisted_resource_entity(&self) -> Option<EntityId> {
        query(()).incl(persistent_resources()).iter(self, None).map(|(id, _)| id).next()
    }
    fn persisted_resource<T: ComponentValue>(&self, component: Component<T>) -> Option<&T> {
        assert_persisted(*component);
        self.persisted_resource_entity().and_then(|id| self.get_ref(id, component).ok())
    }
    fn persisted_resource_mut<T: ComponentValue>(&mut self, component: Component<T>) -> Option<&mut T> {
        assert_persisted(*component);
        self.persisted_resource_entity().and_then(|id| self.get_mut(id, component).ok())
    }

    fn synced_resource_entity(&self) -> Option<EntityId> {
        query(()).incl(synced_resources()).iter(self, None).map(|(id, _)| id).next()
    }
    fn synced_resource<T: ComponentValue>(&self, component: Component<T>) -> Option<&T> {
        assert_networked(*component);
        self.synced_resource_entity().and_then(|id| self.get_ref(id, component).ok())
    }
    fn synced_resource_mut<T: ComponentValue>(&mut self, component: Component<T>) -> Option<&mut T> {
        self.synced_resource_entity().and_then(|id| self.get_mut(id, component).ok())
    }
}

pub fn assert_networked(desc: ambient_ecs::ComponentDesc) {
    if !desc.has_attribute::<Networked>() {
        panic!("Attempt to access sync {desc:#?} which is not marked as `Networked`. Attributes: {:?}", desc.attributes());
    }

    if !desc.has_attribute::<Serializable>() {
        panic!("Sync component {desc:#?} is not serializable. Attributes: {:?}", desc.attributes());
    }
}

fn assert_persisted(desc: ambient_ecs::ComponentDesc) {
    assert_networked(desc);

    if !desc.has_attribute::<Store>() {
        panic!("Attempt to access persisted resource {desc:?} which is not `Store`");
    }
}

pub async fn rpc_request<
    Args: Send + 'static,
    Req: Serialize + DeserializeOwned + Send + 'static,
    Resp: Serialize + DeserializeOwned + Send,
    F: Fn(Args, Req) -> L + Send + Sync + Copy + 'static,
    L: Future<Output = Resp> + Send,
>(
    conn: &Connection,
    reg: Arc<RpcRegistry<Args>>,
    func: F,
    req: Req,
    size_limit: usize,
) -> Result<Resp, NetworkError> {
    let stream = conn.open_bi();
    let (mut send, recv) = stream.await.map_err(NetworkError::ConnectionError)?;
    send.write_u32(RPC_BISTREAM_ID).await?;
    let req = reg.serialize_req(func, req);
    send.write_all(&req).await.map_err(NetworkError::from)?;
    send.finish().await.map_err(NetworkError::from)?;
    drop(send);
    let resp = recv.read_to_end(size_limit).await.map_err(NetworkError::from)?;
    let resp = reg.deserialize_resp(func, &resp)?;
    Ok(resp)
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("No more data to be read from stream")]
    EndOfStream,
    #[error("Connection closed by peer")]
    ConnectionClosed,
    #[error("Bad bincode message format: {0:?}")]
    BadMsgFormat(#[from] bincode::Error),
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
    #[error("Quinn connection failed")]
    ConnectionError(#[from] quinn::ConnectionError),
    #[error(transparent)]
    ReadToEndError(#[from] quinn::ReadToEndError),
    #[error(transparent)]
    WriteError(#[from] quinn::WriteError),
    #[error(transparent)]
    SendDatagramError(#[from] quinn::SendDatagramError),
    #[error(transparent)]
    RpcError(#[from] RpcError),
    // FIXME: change into a more specific ProxyError
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl NetworkError {
    /// Returns true if the connection was properly closed.
    ///
    /// Does not return true if the stream was closed as the connection may
    /// still be alive.
    pub fn is_closed(&self) -> bool {
        match self {
            Self::ConnectionClosed => true,
            // The connection was closed automatically,
            // for example by dropping the [`quinn::Connection`]
            Self::ConnectionError(ConnectionClosed(ConnectionClose { error_code, .. })) if u64::from(*error_code) == 0 => true,
            Self::IOError(err) if matches!(err.kind(), ErrorKind::ConnectionReset) => true,
            _ => false,
        }
    }

    /// Returns `true` if the network error is [`EndOfStream`].
    ///
    /// [`EndOfStream`]: NetworkError::EndOfStream
    #[must_use]
    pub fn is_end_of_stream(&self) -> bool {
        matches!(self, Self::EndOfStream)
    }
}

/// Abstracts the serialization for a fixed size stream.
#[derive(Debug)]
pub struct IncomingStream {
    pub stream: FramedRead<quinn::RecvStream, LengthDelimitedCodec>,
}
impl IncomingStream {
    /// Accept a new uni-directional peer stream. Waits for the server to open a
    /// stream.
    pub async fn accept_incoming(conn: &Connection) -> Result<Self, NetworkError> {
        let stream = conn.accept_uni().await.map_err(NetworkError::from)?;
        Ok(Self::new(stream))
    }

    pub fn new(stream: quinn::RecvStream) -> Self {
        let mut codec = LengthDelimitedCodec::new();
        codec.set_max_frame_length(1_024 * 1_024 * 1_024);
        Self { stream: FramedRead::new(stream, codec) }
    }

    /// Reads the next frame from the incoming stream
    pub async fn next<T: DeserializeOwned + std::fmt::Debug>(&mut self) -> Result<T, NetworkError> {
        let buf = self
            .stream
            .next()
            .await
            // There is nothing more to read from the stream since it was
            // closed by peer
            .ok_or(NetworkError::EndOfStream)?
            // Reading was not possible as the connection was closed
            .map_err(|_| NetworkError::ConnectionClosed)?;

        bincode::deserialize(&buf).map_err(Into::into)
    }
}

#[derive(Debug)]
pub struct OutgoingStream {
    pub stream: FramedWrite<quinn::SendStream, LengthDelimitedCodec>,
}
impl OutgoingStream {
    /// Are you sure you don't want [open_uni_with_id] instead?
    pub async fn open_uni(conn: &Connection) -> Result<Self, NetworkError> {
        Ok(OutgoingStream::new(conn.open_uni().await?))
    }
    pub async fn open_uni_with_id(conn: &Connection, id: u32) -> Result<Self, NetworkError> {
        let mut stream = Self::open_uni(conn).await?;
        stream.stream.get_mut().write_u32(id).await?;
        Ok(stream)
    }

    pub fn new(stream: quinn::SendStream) -> Self {
        let mut codec = LengthDelimitedCodec::new();
        codec.set_max_frame_length(1_024 * 1_024 * 1_024);
        Self { stream: FramedWrite::new(stream, codec) }
    }

    /// Sends raw bytes over the network
    pub async fn send_bytes(&mut self, bytes: Vec<u8>) -> Result<(), NetworkError> {
        self.stream.send(bytes.into()).await?;

        Ok(())
    }

    pub async fn send<T: Serialize>(&mut self, value: &T) -> Result<(), NetworkError> {
        let bytes = bincode::serialize(value)?;
        self.send_bytes(bytes).await
    }
}

/// Are you sure you don't want [open_bincode_bi_stream_with_id] instead?
pub async fn open_bincode_bi_stream(conn: &Connection) -> Result<(OutgoingStream, IncomingStream), NetworkError> {
    let (send, recv) = conn.open_bi().await?;
    Ok((OutgoingStream::new(send), IncomingStream::new(recv)))
}

pub async fn open_bincode_bi_stream_with_id(conn: &Connection, id: u32) -> Result<(OutgoingStream, IncomingStream), NetworkError> {
    let (mut send, recv) = conn.open_bi().await?;
    send.write_u32(id).await?;
    Ok((OutgoingStream::new(send), IncomingStream::new(recv)))
}

pub async fn next_bincode_bi_stream(conn: &Connection) -> Result<(OutgoingStream, IncomingStream), NetworkError> {
    let (send, recv) = conn.accept_bi().await?;
    let send = OutgoingStream::new(send);
    let recv = IncomingStream::new(recv);
    Ok((send, recv))
}

pub fn send_datagram(conn: &Connection, id: u32, mut payload: Vec<u8>) -> Result<(), NetworkError> {
    let mut bytes = Vec::new();
    byteorder::WriteBytesExt::write_u32::<byteorder::BigEndian>(&mut bytes, id)?;
    bytes.append(&mut payload);
    conn.send_datagram(Bytes::from(bytes))?;

    Ok(())
}

pub fn create_client_endpoint_random_port() -> Option<Endpoint> {
    for _ in 0..10 {
        let client_port = {
            let mut rng = rand::thread_rng();
            rng.gen_range(15000..25000)
        };
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), client_port);

        if let Ok(mut endpoint) = Endpoint::client(client_addr) {
            let cert = Certificate(CERT.to_vec());
            let mut roots = RootCertStore::empty();
            roots.add(&cert).unwrap();
            let crypto = rustls::ClientConfig::builder().with_safe_defaults().with_root_certificates(roots).with_no_client_auth();
            let mut transport = TransportConfig::default();
            transport.keep_alive_interval(Some(Duration::from_secs_f32(1.)));
            if std::env::var("AMBIENT_DISABLE_TIMEOUT").is_ok() {
                transport.max_idle_timeout(None);
            } else {
                transport.max_idle_timeout(Some(Duration::from_secs_f32(60.).try_into().unwrap()));
            }
            let mut client_config = ClientConfig::new(Arc::new(crypto));
            client_config.transport_config(Arc::new(transport));

            endpoint.set_default_client_config(client_config);
            return Some(endpoint);
        }
    }
    None
}

fn create_server(server_addr: SocketAddr) -> anyhow::Result<Endpoint> {
    let cert = Certificate(CERT.to_vec());
    let cert_key = PrivateKey(CERT_KEY.to_vec());
    let mut server_conf = ServerConfig::with_single_cert(vec![cert], cert_key)?;
    let mut transport = TransportConfig::default();
    if std::env::var("AMBIENT_DISABLE_TIMEOUT").is_ok() {
        transport.max_idle_timeout(None);
    } else {
        transport.max_idle_timeout(Some(Duration::from_secs_f32(60.).try_into()?));
    }
    server_conf.transport = Arc::new(transport);
    Ok(Endpoint::server(server_conf, server_addr)?)
}

pub const CERT: &[u8] = include_bytes!("./cert.der");
pub const CERT_KEY: &[u8] = include_bytes!("./cert.key.der");

#[macro_export]
macro_rules! log_network_result {
    ( $x:expr ) => {
        if let Err(err) = $x {
            $crate::log_network_error(&err.into());
        }
    };
}

pub fn log_network_error(err: &anyhow::Error) {
    if let Some(quinn::WriteError::ConnectionLost(err)) = err.downcast_ref::<quinn::WriteError>() {
        log::info!("Connection lost: {:#}", err);
    } else if let Some(err) = err.downcast_ref::<quinn::ConnectionError>() {
        log::info!("Connection error: {:#}", err);
    } else if let Some(err) = err.downcast_ref::<quinn::WriteError>() {
        log::info!("Write error: {:#}", err);
    } else {
        log_error(err);
    }
}

#[macro_export]
macro_rules! unwrap_log_network_err {
    ( $x:expr ) => {
        match $x {
            Ok(val) => val,
            Err(err) => {
                $crate::log_network_error(&err.into());
                return Default::default();
            }
        }
    };
}
