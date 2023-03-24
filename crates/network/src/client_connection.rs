use bytes::Bytes;
use futures::StreamExt;
use quinn::{NewConnection, RecvStream, SendStream, ConnectionError};

use crate::{OutgoingStream, next_bincode_bi_stream, IncomingStream, NetworkError};

pub enum ClientConnection {
    Direct(NewConnection),
    // TODO: Proxied(),
}

fn map_to_network_error<T>(val: Result<T, ConnectionError>) -> Result<T, NetworkError> {
    val.map_err(NetworkError::from)
}

fn map_opt_to_network_error<T>(val: Option<Result<T, ConnectionError>>) -> Option<Result<T, NetworkError>> {
    val.map(map_to_network_error)
}

impl ClientConnection {
    pub async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        match self {
            Self::Direct(conn) => map_to_network_error(conn.connection.open_uni().await),
        }
    }

    pub async fn open_bincode_uni(&self) -> Result<OutgoingStream, NetworkError> {
        match self {
            Self::Direct(conn) => OutgoingStream::open_uni(&conn.connection).await,
        }
    }

    pub async fn accept_uni(&self) -> Option<Result<RecvStream, NetworkError>> {
        match self {
            Self::Direct(conn) => map_opt_to_network_error(conn.uni_streams.next().await),
        }
    }

    pub async fn accept_bi(&self) -> Option<Result<(SendStream, RecvStream), NetworkError>> {
        match self {
            Self::Direct(conn) => map_opt_to_network_error(conn.bi_streams.next().await),
        }
    }

    pub async fn accept_bincode_bi(&self) -> Result<(OutgoingStream, IncomingStream), NetworkError> {
        match self {
            Self::Direct(conn) => next_bincode_bi_stream(conn).await,
        }
    }

    pub async fn read_datagram(&self) -> Option<Result<Bytes, NetworkError>> {
        match self {
            Self::Direct(conn) => map_opt_to_network_error(conn.datagrams.next().await),
        }
    }
}

pub enum Datagrams {
    Direct(quinn::Datagrams),
    // TODO: Proxied(),
}

impl From<NewConnection> for ClientConnection {
    fn from(value: NewConnection) -> Self {
        Self::Direct(value)
    }
}
