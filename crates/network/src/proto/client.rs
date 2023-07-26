use std::sync::Arc;

use ambient_ecs::{
    generated::network::components::is_remote_entity, ComponentRegistry, Entity, WorldDiff,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::ContentBaseUrlKey,
};
use anyhow::Context;
use bytes::Bytes;
use parking_lot::Mutex;
use tokio::io::AsyncReadExt;
use tracing::{debug_span, Instrument};

use crate::{
    bytes_ext::BufExt,
    client::{
        bi_stream_handlers, datagram_handlers, uni_stream_handlers, PlatformRecvStream,
        PlatformSendStream,
    },
    client_game_state::ClientGameState,
    log_task_result,
    proto::*,
};

/// The client logic handler in a connected state
///
/// Entered after the client has sent a connect request and received a `ServerInfo` message from the server
#[derive(Debug)]
pub(crate) struct ConnectedClient {}

#[derive(Debug)]
pub(crate) enum ClientProtoState {
    Pending(String),
    Connected(ConnectedClient),
    Disconnected,
}

/// Holds the material world of the client.
pub type SharedClientGameState = Arc<Mutex<ClientGameState>>;

impl ClientProtoState {
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
                ContentBaseUrlKey.insert(assets, server_info.content_base_url.clone());
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
    pub fn process_client_stats(
        &mut self,
        state: &SharedClientGameState,
        stats: crate::client::NetworkStats,
    ) {
        use crate::client::client_network_stats;

        let mut gs = state.lock();
        tracing::debug!(?stats, "Client network stats");
        gs.world.add_resource(client_network_stats(), stats);
    }

    /// Returns `true` if the client state is [`Pending`].
    ///
    /// [`Pending`]: ClientProtoState::Pending
    #[must_use]
    pub(crate) fn is_pending(&self) -> bool {
        matches!(self, Self::Pending(..))
    }

    /// Returns `true` if the client state is [`Connected`].
    ///
    /// [`Connected`]: ClientProtoState::Connected
    #[must_use]
    pub(crate) fn is_connected(&self) -> bool {
        matches!(self, Self::Connected(..))
    }
}

impl ConnectedClient {
    #[tracing::instrument(level = "debug")]
    pub fn process_diff(
        &mut self,
        state: &SharedClientGameState,
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

    /// Processes an incoming datagram
    #[tracing::instrument(level = "debug")]
    pub fn process_datagram(
        &mut self,
        state: &SharedClientGameState,
        mut data: Bytes,
    ) -> anyhow::Result<()> {
        let id = data.try_get_u32()?;

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

    /// Processes a server initiated unidirectional stream
    pub fn process_uni(&mut self, state: &SharedClientGameState, mut recv: PlatformRecvStream) {
        let state = state.clone();

        let task = log_task_result(
            async move {
                let id = recv.read_u32().await?;

                // The handler is returned to avoid holding the lock while the handler is running
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

                    handler(world, assets, recv).instrument(tracing::debug_span!(
                        "handle_uni",
                        name,
                        id
                    ))
                };

                handler.await;

                Ok(())
            }
            .instrument(debug_span!("process_uni")),
        );

        let rt = ambient_sys::task::RuntimeHandle::current();
        #[cfg(target_os = "unknown")]
        rt.spawn_local(task);
        #[cfg(not(target_os = "unknown"))]
        rt.spawn(task);
    }

    /// Processes a server initiated bidirectional stream
    pub fn process_bi(
        &mut self,
        state: &SharedClientGameState,
        send: PlatformSendStream,
        mut recv: PlatformRecvStream,
    ) {
        let state = state.clone();

        let task = log_task_result(async move {
            let id = recv.read_u32().await?;

            // The handler is returned to avoid holding the lock while the handler is running
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
        })
        .instrument(debug_span!("process_bi"));

        let rt = ambient_sys::task::RuntimeHandle::current();
        #[cfg(target_os = "unknown")]
        rt.spawn_local(task);
        #[cfg(not(target_os = "unknown"))]
        rt.spawn(task);
    }
}
