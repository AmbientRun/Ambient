use ambient_proxy::client::ProxiedConnection;
use async_trait::async_trait;
use bytes::Bytes;
use quinn::{Connection, RecvStream, SendStream};

use crate::NetworkError;

/// Incoming quinn connection from the client that can be either direct or proxied
#[derive(Debug, Clone)]
pub enum ConnectionInner {
    Direct(Connection),
    Proxied(ProxiedConnection),
}

impl From<Connection> for ConnectionInner {
    fn from(value: Connection) -> Self {
        Self::Direct(value)
    }
}

impl From<ProxiedConnection> for ConnectionInner {
    fn from(value: ProxiedConnection) -> Self {
        Self::Proxied(value)
    }
}

impl ConnectionInner {
    #[inline]
    pub async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        match self {
            ConnectionInner::Direct(conn) => Ok(conn.open_uni().await?),
            ConnectionInner::Proxied(conn) => Ok(conn.open_uni().await?),
        }
    }

    #[inline]
    pub async fn open_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        match self {
            ConnectionInner::Direct(conn) => Ok(conn.open_bi().await?),
            ConnectionInner::Proxied(conn) => Ok(conn.open_bi().await?),
        }
    }

    #[inline]
    pub async fn accept_uni(&self) -> Result<RecvStream, NetworkError> {
        match self {
            ConnectionInner::Direct(conn) => Ok(conn.accept_uni().await?),
            ConnectionInner::Proxied(conn) => Ok(conn.accept_uni().await),
        }
    }

    #[inline]
    pub async fn accept_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        match self {
            ConnectionInner::Direct(conn) => Ok(conn.accept_bi().await?),
            ConnectionInner::Proxied(conn) => Ok(conn.accept_bi().await),
        }
    }

    #[inline]
    pub async fn read_datagram(&self) -> Result<Bytes, NetworkError> {
        match self {
            ConnectionInner::Direct(conn) => Ok(conn.read_datagram().await?),
            ConnectionInner::Proxied(conn) => Ok(conn.read_datagram().await),
        }
    }

    #[inline]
    pub async fn send_datagram(&self, data: Bytes) -> Result<(), NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.send_datagram(data)?),
            ClientConnection::Proxied(conn) => Ok(conn.send_datagram(data)?),
        }
    }
}
