use bytes::Bytes;
use quinn::{RecvStream, SendStream, ConnectionError, Connection};

use crate::{OutgoingStream, next_bincode_bi_stream, IncomingStream, NetworkError};

#[derive(Debug, Clone)]
pub enum ClientConnection {
    Direct(Connection),
    // TODO: Proxied(),
}

fn map_to_network_error<T>(val: Result<T, ConnectionError>) -> Result<T, NetworkError> {
    val.map_err(NetworkError::from)
}

impl ClientConnection {
    pub fn fixme_unwrap(&self) -> Connection {
        let Self::Direct(conn) = self;
        conn.clone()
    }

    pub async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        match self {
            Self::Direct(conn) => map_to_network_error(conn.open_uni().await),
        }
    }

    pub async fn open_bincode_uni(&self) -> Result<OutgoingStream, NetworkError> {
        match self {
            Self::Direct(conn) => OutgoingStream::open_uni(&conn).await,
        }
    }

    pub async fn accept_uni(&self) -> Result<RecvStream, NetworkError> {
        match self {
            Self::Direct(conn) => map_to_network_error(conn.accept_uni().await),
        }
    }

    pub async fn accept_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        match self {
            Self::Direct(conn) => map_to_network_error(conn.accept_bi().await),
        }
    }

    pub async fn accept_bincode_bi(&self) -> Result<(OutgoingStream, IncomingStream), NetworkError> {
        match self {
            Self::Direct(conn) => next_bincode_bi_stream(conn).await,
        }
    }

    pub async fn read_datagram(&self) -> Result<Bytes, NetworkError> {
        match self {
            Self::Direct(conn) => map_to_network_error(conn.read_datagram().await),
        }
    }
}

impl From<Connection> for ClientConnection {
    fn from(value: Connection) -> Self {
        Self::Direct(value)
    }
}
