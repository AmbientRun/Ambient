use ambient_proxy::client::ProxiedConnection;
use bytes::Bytes;
use quinn::{Connection, RecvStream, SendStream};

use crate::{next_bincode_bi_stream, IncomingStream, NetworkError, OutgoingStream};

#[derive(Debug, Clone)]
pub enum ClientConnection {
    Direct(Connection),
    Proxied(ProxiedConnection),
}

impl ClientConnection {
    pub fn fixme_unwrap(&self) -> Connection {
        let Self::Direct(conn) = self else { panic!("Not a direct connection") };
        conn.clone()
    }

    pub async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.open_uni().await?),
            ClientConnection::Proxied(conn) => Ok(conn.open_uni().await?),
        }
    }

    pub async fn open_bincode_uni(&self) -> Result<OutgoingStream, NetworkError> {
        match self {
            ClientConnection::Direct(conn) => OutgoingStream::open_uni(&conn).await,
            ClientConnection::Proxied(conn) => Ok(OutgoingStream::new(conn.open_uni().await?)),
        }
    }

    pub async fn accept_uni(&self) -> Result<RecvStream, NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.accept_uni().await?),
            ClientConnection::Proxied(conn) => Ok(conn.accept_uni().await),
        }
    }

    pub async fn accept_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.accept_bi().await?),
            ClientConnection::Proxied(conn) => Ok(conn.accept_bi().await),
        }
    }

    pub async fn accept_bincode_bi(&self) -> Result<(OutgoingStream, IncomingStream), NetworkError> {
        match self {
            ClientConnection::Direct(conn) => next_bincode_bi_stream(conn).await,
            ClientConnection::Proxied(conn) => {
                let (tx, rx) = conn.accept_bi().await;
                Ok((OutgoingStream::new(tx), IncomingStream::new(rx)))
            }
        }
    }

    pub async fn read_datagram(&self) -> Result<Bytes, NetworkError> {
        match self {
            ClientConnection::Direct(conn) => Ok(conn.read_datagram().await?),
            ClientConnection::Proxied(conn) => Ok(conn.read_datagram().await),
        }
    }
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
