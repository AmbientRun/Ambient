use ambient_proxy::client::ProxiedConnection;
use bytes::Bytes;
use quinn::{Connection, RecvStream, SendStream};

use crate::NetworkError;

/// Incoming quinn connection from the client that can be either direct or proxied
#[derive(Debug, Clone)]
pub enum ConnectionKind {
    Direct(Connection),
    Proxied(ProxiedConnection),
}

impl From<Connection> for ConnectionKind {
    fn from(value: Connection) -> Self {
        Self::Direct(value)
    }
}

impl From<ProxiedConnection> for ConnectionKind {
    fn from(value: ProxiedConnection) -> Self {
        Self::Proxied(value)
    }
}

impl ConnectionKind {
    pub fn is_local(&self) -> bool {
        match self {
            ConnectionKind::Direct(conn) => conn.remote_address().ip().is_loopback(),
            ConnectionKind::Proxied(_) => false,
        }
    }

    #[inline]
    pub async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        match self {
            ConnectionKind::Direct(conn) => Ok(conn.open_uni().await?),
            ConnectionKind::Proxied(conn) => Ok(conn.open_uni().await?),
        }
    }

    #[inline]
    pub async fn open_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        match self {
            ConnectionKind::Direct(conn) => Ok(conn.open_bi().await?),
            ConnectionKind::Proxied(conn) => Ok(conn.open_bi().await?),
        }
    }

    #[inline]
    pub async fn accept_uni(&self) -> Result<RecvStream, NetworkError> {
        match self {
            ConnectionKind::Direct(conn) => Ok(conn.accept_uni().await?),
            ConnectionKind::Proxied(conn) => Ok(conn.accept_uni().await),
        }
    }

    #[inline]
    pub async fn accept_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        match self {
            ConnectionKind::Direct(conn) => Ok(conn.accept_bi().await?),
            ConnectionKind::Proxied(conn) => Ok(conn.accept_bi().await),
        }
    }

    #[inline]
    pub async fn read_datagram(&self) -> Result<Bytes, NetworkError> {
        match self {
            ConnectionKind::Direct(conn) => Ok(conn.read_datagram().await?),
            ConnectionKind::Proxied(conn) => Ok(conn.read_datagram().await),
        }
    }

    #[inline]
    pub fn send_datagram(&self, data: Bytes) -> Result<(), NetworkError> {
        match self {
            ConnectionKind::Direct(conn) => Ok(conn.send_datagram(data)?),
            ConnectionKind::Proxied(conn) => Ok(conn.send_datagram(data)?),
        }
    }
}
