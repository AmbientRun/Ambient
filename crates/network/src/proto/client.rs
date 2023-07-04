use std::sync::Arc;

use ambient_ecs::{
    generated::components::core::network::is_remote_entity, ComponentRegistry, Entity, WorldDiff,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::ContentBaseUrlKey,
};
use anyhow::{bail, Context};
use bytes::{Buf, Bytes};
use parking_lot::Mutex;
use tokio::io::AsyncReadExt;
use tracing::{debug_span, Instrument};

use crate::{
    client::{
        bi_stream_handlers, client_network_stats, datagram_handlers, uni_stream_handlers,
        NetworkStats, PlatformRecvStream, PlatformSendStream,
    },
    client_game_state::ClientGameState,
    proto::*,
};

/// The client logic handler in a connected state
///
/// Entered after the client has sent a connect request and received a `ServerInfo` message from the server, in no particular order.
#[derive(Debug)]
pub(crate) struct ConnectedClient {}

#[derive(Debug)]
pub(crate) enum ClientState {
    Pending(String),
    Connected(ConnectedClient),
    Disconnected,
}

/// Holds the material world of the client.
pub type SharedClientState = Arc<Mutex<ClientGameState>>;

impl ClientState {
    pub fn process_disconnect(&mut self) {
        tracing::info!("Disconnecting client: {self:#?}");

        *self = Self::Disconnected;
    }

    /// Processes an incoming control frame from the server.
    #[tracing::instrument(level = "debug")]
    pub fn process_push(&mut self, assets: &AssetCache, frame: ServerPush) -> anyhow::Result<()> {
        match (frame, &self) {
            (ServerPush::ServerInfo(server_info), Self::Pending(_user_id)) => {
                let current_version = get_version_with_revision();

                if server_info.version != current_version {
                    tracing::error!(
                        "Client version does not match server version. Server version: {:?}, Client version {:?}",
                        server_info.version,
                        current_version
                    );
                }

                tracing::debug!(content_base_url=?server_info.content_base_url, "Inserting content base url");
                ContentBaseUrlKey.insert(&assets, server_info.content_base_url.clone());
                tracing::debug!(?server_info.external_components, "Adding external components");
                ComponentRegistry::get_mut().add_external(server_info.external_components);

                *self = Self::Connected(ConnectedClient {});

                Ok(())
            }
            (ServerPush::ServerInfo(_), _) => {
                tracing::warn!("Received server info while already connected");
                Ok(())
            }
            (ServerPush::Disconnect, _) => {
                self.process_disconnect();
                Ok(())
            }
        }
    }

    #[cfg(not(target_os = "unknown"))]
    pub fn process_client_stats(&mut self, state: &SharedClientState, stats: NetworkStats) {
        let mut gs = state.lock();
        tracing::debug!(?stats, "Client network stats");
        gs.world.add_resource(client_network_stats(), stats);
    }

    /// Returns `true` if the client state is [`Pending`].
    ///
    /// [`Pending`]: ClientState::Pending
    #[must_use]
    pub(crate) fn is_pending(&self) -> bool {
        matches!(self, Self::Pending(..))
    }

    /// Returns `true` if the client state is [`Connected`].
    ///
    /// [`Connected`]: ClientState::Connected
    #[must_use]
    pub(crate) fn is_connected(&self) -> bool {
        matches!(self, Self::Connected(..))
    }
}

impl ConnectedClient {
    #[tracing::instrument(level = "debug")]
    pub fn process_diff(
        &mut self,
        state: &SharedClientState,
        diff: WorldDiff,
    ) -> anyhow::Result<()> {
        let mut gs = state.lock();
        tracing::debug!(diff=?diff.len(), "Applying diff");
        diff.apply(
            &mut gs.world,
            Entity::new().with(is_remote_entity(), ()),
            false,
        );
        Ok(())
    }

    /// Processes a server initiated bidirectional stream
    pub async fn process_bi(
        &mut self,
        state: &SharedClientState,
        send: PlatformSendStream,
        mut recv: PlatformRecvStream,
    ) -> anyhow::Result<()> {
        let id = recv.read_u32().await?;

        let handler = {
            let mut gs = state.lock();
            let gs = &mut *gs;
            let world = &mut gs.world;
            let assets = gs.assets.clone();

            let (name, handler) = world
                .resource(bi_stream_handlers())
                .get(&id)
                .with_context(|| format!("No handler for stream {id}"))?
                .clone();

            handler(world, assets, send, recv).instrument(debug_span!("handle_bi", name, id))
        };

        handler.await;

        Ok(())
    }

    /// Processes a server initiated unidirectional stream
    pub async fn process_uni(
        &mut self,
        state: &SharedClientState,
        mut recv: PlatformRecvStream,
    ) -> anyhow::Result<()> {
        let id = recv.read_u32().await?;
        let handler = {
            let mut gs = state.lock();
            let gs = &mut *gs;
            let world = &mut gs.world;
            let assets = gs.assets.clone();

            let (name, handler) = world
                .resource(uni_stream_handlers())
                .get(&id)
                .with_context(|| format!("No handler for stream {id}"))?
                .clone();

            handler(world, assets, recv).instrument(tracing::debug_span!("handle_uni", name, id))
        };

        handler.await;

        Ok(())
    }

    /// Processes an incoming datagram
    #[tracing::instrument(level = "debug")]
    pub fn process_datagram(
        &mut self,
        state: &SharedClientState,
        mut data: Bytes,
    ) -> anyhow::Result<()> {
        if data.len() < 4 {
            bail!("Received malformed datagram");
        }

        let id = data.get_u32();

        let mut gs = state.lock();
        let gs = &mut *gs;
        let world = &mut gs.world;
        let assets = gs.assets.clone();

        let (name, handler) = world
            .resource(datagram_handlers())
            .get(&id)
            .with_context(|| format!("No handler for stream {id}"))?
            .clone();

        let _span = debug_span!("handle_uni", name, id).entered();
        handler(world, assets, data);

        Ok(())
    }
}
