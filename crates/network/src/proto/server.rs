use anyhow::{bail, Context};
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
    pub async fn process_control(&mut self, state: &SharedServerState, frame: ServerControl) -> anyhow::Result<()> {
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
                self.process_disconnect(state);
                Ok(())
            }
        }
    }

    fn process_connect(&mut self, user_id: String) {
        log::debug!("[{}] Locking world", user_id);
        let mut state = state.lock();
        // If there's an old player send a graceful disconnect
        let reconnecting = if let Some(player) = state.players.get_mut(user_id) {
            player.control.send(ClientControl::Disconnect).ok();

            player.control = control_tx;
            player.connection_id = connection_id.clone();
            log::debug!("[{}] Player reconnecting", user_id);
            true
        } else {
            state.players.insert(
                user_id.clone(),
                Player { instance: MAIN_INSTANCE_ID.to_string(), control: control_tx, connection_id: connection_id.clone() },
            );
            false
        };

        let instance = state.instances.get_mut(MAIN_INSTANCE_ID).unwrap();

        // Bring world stream up to the current time
        log::debug!("[{}] Broadcasting diffs", user_id);
        instance.broadcast_diffs();
        log::debug!("[{}] Creating init diff", user_id);

        let diff = world_stream_filter.initial_diff(&instance.world);
        let diff = bincode::serialize(&diff).unwrap();

        log_result!(diffs_tx.send(diff));
        log::debug!("[{}] Init diff sent", user_id);

        if !reconnecting {
            instance.spawn_player(
                create_player_entity_data(user_id, diffs_tx.clone(), stats_tx.clone())
                    .with(player_connection(), new_player_connection.clone()),
            );
            log::info!("[{}] Player spawned", user_id);
        } else {
            let entity = get_by_user_id(&instance.world, user_id).unwrap();
            instance.world.set(entity, player_entity_stream(), diffs_tx.clone()).unwrap();
            instance.world.set(entity, player_stats_stream(), stats_tx.clone()).unwrap();
            instance.world.set(entity, player_connection(), new_player_connection.clone()).unwrap();
            log::info!("[{}] Player reconnected", user_id);
        }
    }

    pub fn process_disconnect(&mut self, state: &SharedServerState) {
        tracing::info!("Client wants to disconnect");
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
    pub async fn process_datagram(&mut self, state: &SharedServerState, mut data: Bytes) -> anyhow::Result<()> {
        if data.len() < 4 {
            bail!("Received malformed datagram");
        }

        let id = data.get_u32();

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
            handler(state.clone(), assets, &self.user_id, data);
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
