pub mod client;

use std::{sync::Arc, time::Duration};

use ambient_core::{asset_cache, gpu};
use ambient_ecs::{world_events, Entity, SystemGroup};
use ambient_element::{Element, ElementComponent, Hooks};
use ambient_renderer::RenderTarget;
use ambient_rpc::RpcRegistry;
use ambient_std::{asset_cache::AssetCache, cb, Cb};
use ambient_sys::{time::interval, MissedTickBehavior};
use anyhow::Context;
use bytes::Bytes;
use flume::Sender;
use futures::{future::BoxFuture, SinkExt, StreamExt};
use glam::{uvec2, uvec4};
use parking_lot::Mutex;
use tokio::io::AsyncReadExt;
use tokio::sync::oneshot;
use url::Url;

use crate::{
    client::{ClientConnection, Control, GameClient, GameClientRenderTarget, LoadedFunc},
    client_game_state::ClientGameState,
    log_network_result,
    proto::{
        client::{ClientState, SharedClientState},
        ClientRequest,
    },
    server::RpcArgs,
    stream::{self, FramedRecvStream, FramedSendStream},
    webtransport::{self, Connection},
    NetworkError,
};

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

impl ClientConnection for WebTransportProxy {
    fn request_bi(&self, id: u32, data: Bytes) -> BoxFuture<Result<Bytes, NetworkError>> {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();

            self.tx
                .send_async(ProxyMessage::RequestBi { id, data, resp: tx })
                .await
                .map_err(|_| NetworkError::ConnectionClosed)?;

            rx.await.map_err(|_| NetworkError::ConnectionClosed)
        })
    }

    fn request_uni(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        Box::pin(async move {
            self.tx
                .send_async(ProxyMessage::RequestUni { id, data })
                .await
                .map_err(|_| NetworkError::ConnectionClosed)?;

            Ok(())
        })
    }

    fn send_datagram(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        Box::pin(async move {
            self.tx
                .send_async(ProxyMessage::Datagram { id, data })
                .await
                .map_err(|_| NetworkError::ConnectionClosed)?;

            Ok(())
        })
    }
}
