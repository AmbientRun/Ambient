pub mod client;

use std::sync::atomic::AtomicU32;

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
    next_id: AtomicU32,
    tx: Sender<ProxyMessage>,
}

impl WebTransportProxy {
    pub(crate) fn new(tx: Sender<ProxyMessage>) -> Self {
        Self {
            tx,
            next_id: AtomicU32::new(0),
        }
    }
}

pub(crate) enum ProxyMessage {
    RequestBi {
        message_id: u32,
        id: u32,
        data: Bytes,
        resp: oneshot::Sender<Bytes>,
    },
    RequestUni {
        message_id: u32,
        id: u32,
        data: Bytes,
    },
    Datagram {
        message_id: u32,
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
        let message_id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();

            send_with_backpressure(
                &self.tx,
                ProxyMessage::RequestBi {
                    id,
                    data,
                    resp: tx,
                    message_id,
                },
            )?;
            // self.tx
            //     .send_async(ProxyMessage::RequestBi { id, data, resp: tx })
            //     .await
            //     .map_err(|_| NetworkError::ConnectionClosed)?;

            rx.await.map_err(|_| NetworkError::ConnectionClosed)
        })
    }

    fn request_uni(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        let message_id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move {
            send_with_backpressure(
                &self.tx,
                ProxyMessage::RequestUni {
                    id,
                    data,
                    message_id,
                },
            )?;

            Ok(())
        })
    }

    fn send_datagram(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        let message_id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move {
            send_with_backpressure(
                &self.tx,
                ProxyMessage::Datagram {
                    id,
                    data,
                    message_id,
                },
            )?;
            // self.tx
            //     .send_async(ProxyMessage::Datagram { id, data })
            //     .await
            //     .map_err(|_| NetworkError::ConnectionClosed)?;

            Ok(())
        })
    }
}
