use anyhow::Context;
use bytes::{Buf, Bytes};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tracing::info_span;

use crate::{
    server::{datagram_handlers, uni_stream_handlers, SharedServerState},
    NetworkError,
};

use super::ClientControlFrame;

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
    pub async fn process_control(&mut self, frame: ClientControlFrame) -> anyhow::Result<()> {
        match (frame, &self) {
            (_, Self::Disconnected) => {
                tracing::info!("Client is disconnected, ignoring control frame");
                Ok(())
            }
            (ClientControlFrame::Connect(user_id), Self::PendingConnection) => {
                // Connect the user
                tracing::info!("User connected");
                *self = Self::Connected(ConnectedClient { user_id });
                Ok(())
            }
            (ClientControlFrame::Connect(_), Self::Connected(_)) => {
                tracing::warn!("Client already connected");
                Ok(())
            }
            (ClientControlFrame::Disconnect, _) => {
                tracing::info!("Client wants to disconnect");
                *self = Self::Disconnected;
                Ok(())
            }
        }
    }

    pub fn disconnect(&mut self, state: &SharedServerState) {
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

        let handler = {
            let state = state.lock();
            let world = state.get_player_world(&self.user_id).context("Failed to get player world")?;

            world.resource(datagram_handlers()).get(&id).context("No handler for datagram: {id}")?.clone()
        };

        {
            let _span = info_span!("handle_datagram", id = id);
            handler(state.clone(), todo!(), &self.user_id, datagram);
        }

        Ok(())
    }

    #[tracing::instrument(level = "info", skip(state, stream))]
    pub async fn process_uni(&mut self, state: &SharedServerState, mut stream: impl AsyncRead + Unpin) -> anyhow::Result<()> {
        let id = stream.read_u32().await?;

        let handler = {
            let mut state = state.lock();
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

    #[tracing::instrument(level = "info", skip(state, send, recv))]
    pub async fn process_bi(
        &mut self,
        state: &SharedServerState,
        send: impl AsyncWrite + Unpin,
        recv: impl AsyncRead + Unpin,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
