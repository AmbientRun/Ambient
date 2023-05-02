use anyhow::Context;
use bytes::{Buf, Bytes};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tracing::info_span;

use crate::{
    server::SharedServerState,
    server::{bi_stream_handlers, datagram_handlers, uni_stream_handlers},
};

use super::ServerControl;

/// The server can be in multiple states depending on what has been received from the client.
///
/// The server starts in the `PendingConnection` state, until
/// the clients sends a `Connect` request.
#[derive(Default, Debug)]
pub enum ServerState {
    #[default]
    PendingConnection,
    Connected(ConnectedClient),
    Disconnected,
}

#[derive(Default, Debug, Clone)]
pub struct PendingConnection {}

#[derive(Debug, Clone)]
pub struct ConnectedClient {
    user_id: String,
}

impl ServerState {
    /// Processes a client request
    pub async fn process_control(&mut self, frame: ServerControl) -> anyhow::Result<()> {
        match (frame, &self) {
            (_, Self::Disconnected) => {
                tracing::info!("Client is disconnected, ignoring control frame");
                Ok(())
            }
            (ServerControl::Connect(user_id), Self::PendingConnection) => {
                // Connect the user
                tracing::info!("User connected");
                *self = Self::Connected(ConnectedClient { user_id });
                Ok(())
            }
            (ServerControl::Connect(_), Self::Connected(_)) => {
                tracing::warn!("Client already connected");
                Ok(())
            }
            (ServerControl::Disconnect, _) => {
                tracing::info!("Client wants to disconnect");
                *self = Self::Disconnected;
                Ok(())
            }
        }
    }

    pub fn process_disconnect(&mut self, state: &SharedServerState) {
        if let Self::Connected(ConnectedClient { user_id }) = self {
            tracing::info!(?user_id, "User disconnected");
            let mut state = state.lock();

            if let Some(player) = state.players.remove(user_id) {
                tracing::debug!("Despawning the player from world: {:?}", player.instance);
                state.instances.get_mut(&player.instance).unwrap().despawn_player(&user_id);
            }
        } else {
            tracing::warn!("Tried to disconnect a client that was not connected");
        }

        *self = Self::Disconnected;
    }
}

impl ConnectedClient {
    /// Processes an incoming datagram
    #[tracing::instrument(level = "info", skip(state))]
    pub async fn process_datagram(&mut self, state: &SharedServerState, mut datagram: Bytes) -> anyhow::Result<()> {
        let id = datagram.get_u32();

        tracing::info!(?id, "Received datagram");

        let (handler, assets) = {
            let mut state = state.lock();
            let world = state.get_player_world_mut(&self.user_id).context("Failed to get player world")?;
            (
                world.resource(datagram_handlers()).get(&id).with_context(|| format!("No handler for datagram: {id}"))?.clone(),
                state.assets.clone(),
            )
        };

        {
            let _span = info_span!("handle_datagram", id).entered();
            handler(state.clone(), assets, &self.user_id, datagram);
        }

        Ok(())
    }

    #[tracing::instrument(level = "info", skip(state, stream))]
    pub async fn process_uni<R>(&mut self, state: &SharedServerState, mut stream: R) -> anyhow::Result<()>
    where
        R: 'static + Send + Sync + AsyncRead + Unpin,
    {
        let id = stream.read_u32().await?;

        let (handler, assets) = {
            let mut state = state.lock();
            let world = state.get_player_world_mut(&self.user_id).context("Failed to get player world")?;
            (
                world.resource(uni_stream_handlers()).get(&id).with_context(|| format!("No handler for uni stream: {id}"))?.clone(),
                state.assets.clone(),
            )
        };
        {
            let _span = info_span!("handle_datagram", id).entered();
            handler(state.clone(), assets, &self.user_id, Box::pin(stream));
        }

        Ok(())
    }

    #[tracing::instrument(level = "info", skip(state, send, recv))]
    pub async fn process_bi<S, R>(&mut self, state: &SharedServerState, send: S, mut recv: R) -> anyhow::Result<()>
    where
        R: 'static + Send + Sync + Unpin + AsyncRead,
        S: 'static + Send + Sync + Unpin + AsyncWrite,
    {
        let id = recv.read_u32().await?;

        let (handler, assets) = {
            let mut state = state.lock();
            let world = state.get_player_world_mut(&self.user_id).context("Failed to get player world")?;
            (
                world.resource(bi_stream_handlers()).get(&id).with_context(|| format!("No handler for bi stream: {id}"))?.clone(),
                state.assets.clone(),
            )
        };
        {
            let _span = info_span!("handle_bi", id).entered();
            handler(state.clone(), assets, &self.user_id, Box::pin(send), Box::pin(recv));
        }

        Ok(())
    }
}
