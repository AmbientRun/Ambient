use std::sync::Arc;

use ambient_std::asset_cache::AssetCache;
use anyhow::Context;
use bytes::{Buf, Bytes, BytesMut};
use flume::r#async::RecvStream;
use futures::{Stream, StreamExt};
use parking_lot::Mutex;
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::info_span;

use crate::{
    protocol::ClientInfo,
    server::{datagram_handlers, uni_stream_handlers, ServerState, SharedServerState},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerRequest {
    /// Connect to the server with the specified user id
    RequestConnect(String),
    /// Peer wants to disconnect
    Disconnect,
}

pub enum ServerResponse {
    Ok,
    AlreadyConnected,
    Denied,
}

/// Handles the server side protocol logic.
pub enum Server {
    PendingConnection(PendingConnection),
    Connected(ConnectedClient),
    Disconnected,
}

pub struct PendingConnection {
    state: SharedServerState,
}

pub struct ConnectedClient {
    state: SharedServerState,
    user_id: String,
}

pub struct Server {
    // conn_state: ConnectionState,
    // assets: AssetCache,
}

impl Server {
    pub fn new(state: Arc<Mutex<ServerState>>, assets: AssetCache) -> Self {
        Self { conn_state: ConnectionState::PendingConnection(PendingConnection { state }), assets }
    }

    /// Processes a client request
    pub async fn process_request(&mut self, frame: ServerRequest) -> ServerResponse {
        match frame {
            ServerRequest::RequestConnect(user_id) => {
                // Connect the user
                if self.user_id.is_some() {
                    tracing::error!(?user_id, existing = ?self.user_id, "User already connected");
                    ServerResponse::AlreadyConnected
                } else {
                    tracing::info!("User connected");
                    self.user_id = Some(user_id);
                    ServerResponse::Ok
                }
            }
            ServerRequest::Disconnect => {
                tracing::info!("Client wants to disconnect");
                self.user_id = None;
                ServerResponse::Ok
            }
        }
    }
}

impl ConnectedClient {
    /// Processes an incoming datagram
    #[tracing::instrument(level = "info", skip(self))]
    pub async fn process_datagram(&mut self, mut datagram: Bytes) -> anyhow::Result<()> {
        let id = datagram.get_u32();

        tracing::info!("Received datagram {id}");

        let handler = {
            let state = self.state.lock();
            let world = state.get_player_world(&self.user_id).context("Failed to get player world")?;

            world.resource(datagram_handlers()).get(&id).context("No handler for datagram: {id}")?.clone()
        };

        {
            let _span = info_span!("handle_datagram", id = id);
            handler(self.state.clone(), self.assets.clone(), &self.user_id, datagram);
        }

        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all)]
    pub async fn process_uni<E: Into<anyhow::Error>>(&mut self, mut stream: impl AsyncRead + Unpin) -> anyhow::Result<()> {
        let id = stream.read_u32().await?;

        let handler = {
            let mut state = self.state.lock();
            let world = state.get_player_world(&self.user_id).context("Failed to get player world")?;

            world.resource(uni_stream_handlers()).get(&id).context("No handler for datagram: {id}")?.clone()
        };
        {
            let _span = info_span!("handle_datagram", id = id);
            todo!()
            // handler(self.state.clone(), self.assets.clone(), &self.user_id, stream);
        }

        Ok(())
    }
}
