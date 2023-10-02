pub mod client;

use bytes::Bytes;
use flume::Sender;
use futures::future::BoxFuture;
use tokio::sync::oneshot;

use crate::{client::NetworkTransport, NetworkError};

/// A proxy for the webtransport connection.
///
/// The webtransport connection is due to holding pointers to Js objects and is thus not thread
/// safe.
pub struct WebTransportProxy {
    tx: Sender<ProxyMessage>,
}

impl WebTransportProxy {
    pub(crate) fn new(tx: Sender<ProxyMessage>) -> Self {
        Self { tx }
    }
}

pub(crate) enum ProxyMessage {
    RequestBi {
        id: u32,
        data: Bytes,
        resp: oneshot::Sender<Bytes>,
    },
    RequestUni {
        id: u32,
        data: Bytes,
    },
    Datagram {
        id: u32,
        data: Bytes,
    },
}

fn send_with_backpressure(
    tx: &Sender<ProxyMessage>,
    msg: ProxyMessage,
) -> Result<(), NetworkError> {
    match tx.try_send(msg) {
        Ok(()) => Ok(()),
        Err(flume::TrySendError::Full(_)) => {
            tracing::error!("WebTransportProxy::send_with_backpressure: full");
            Err(NetworkError::Backpressure)
        }
        Err(flume::TrySendError::Disconnected(_)) => {
            tracing::error!("WebTransportProxy::send_with_backpressure: disconnected");

            Err(NetworkError::ConnectionClosed)
        }
    }
}

impl NetworkTransport for WebTransportProxy {
    fn request_bi(&self, id: u32, data: Bytes) -> BoxFuture<Result<Bytes, NetworkError>> {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();

            send_with_backpressure(&self.tx, ProxyMessage::RequestBi { id, data, resp: tx })?;
            // self.tx
            //     .send_async(ProxyMessage::RequestBi { id, data, resp: tx })
            //     .await
            //     .map_err(|_| NetworkError::ConnectionClosed)?;

            rx.await.map_err(|_| NetworkError::ConnectionClosed)
        })
    }

    fn request_uni(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        Box::pin(async move {
            tracing::info!("Sending {data:?}");
            send_with_backpressure(&self.tx, ProxyMessage::RequestUni { id, data })?;

            Ok(())
        })
    }

    fn send_datagram(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        Box::pin(async move {
            tracing::info!("Datagram {data:?}");
            send_with_backpressure(&self.tx, ProxyMessage::Datagram { id, data })?;
            // self.tx
            //     .send_async(ProxyMessage::Datagram { id, data })
            //     .await
            //     .map_err(|_| NetworkError::ConnectionClosed)?;

            Ok(())
        })
    }
}
