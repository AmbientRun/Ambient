use ambient_proxy::client::ProxiedConnection;
use async_trait::async_trait;
use bytes::Bytes;
use quinn::{Connection, RecvStream, SendStream};

use crate::NetworkError;

/// Incoming connection from the client that can be either direct or proxied
#[derive(Debug, Clone)]
pub enum ClientConnection {
    Direct(Connection),
    Proxied(ProxiedConnection),
}

impl From<Connection> for ClientConnection {
    fn from(value: Connection) -> Self {
        Self::Direct(value)
    }
}

impl From<ProxiedConnection> for ClientConnection {
    fn from(value: ProxiedConnection) -> Self {
        Self::Proxied(value)
    }
}

#[async_trait]
impl crate::connection::Connection for ClientConnection {
    async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.open_uni().await?),
            ClientConnection::Proxied(conn) => Ok(conn.open_uni().await?),
        }
    }

    async fn open_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.open_bi().await?),
            ClientConnection::Proxied(conn) => Ok(conn.open_bi().await?),
        }
    }

    async fn accept_uni(&self) -> Result<RecvStream, NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.accept_uni().await?),
            ClientConnection::Proxied(conn) => Ok(conn.accept_uni().await),
        }
    }

    async fn accept_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.accept_bi().await?),
            ClientConnection::Proxied(conn) => Ok(conn.accept_bi().await),
        }
    }

    async fn read_datagram(&self) -> Result<Bytes, NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.read_datagram().await?),
            ClientConnection::Proxied(conn) => Ok(conn.read_datagram().await),
        }
    }

    fn send_datagram(&self, data: Bytes) -> Result<(), NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.send_datagram(data)?),
            ClientConnection::Proxied(conn) => Ok(conn.send_datagram(data)?),
        }
    }
}
