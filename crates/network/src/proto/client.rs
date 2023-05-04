use std::sync::Arc;

use ambient_ecs::{generated::components::core::network::is_remote_entity, Entity, WorldDiff};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::ContentBaseUrlKey,
    fps_counter::FpsSample,
    Cb,
};
use anyhow::{bail, Context};
use bytes::{Buf, Bytes};
use parking_lot::Mutex;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tracing::info_span;

use crate::{
    client::{bi_stream_handlers, datagram_handlers, uni_stream_handlers, GameClientServerStats},
    client_game_state::ClientGameState,
    proto::*,
    protocol::ServerInfo,
};

/// The client logic handler in a connected state
///
/// Entered after the client has sent a connect request and received a `ServerInfo` message from the server, in no particular order.
#[derive(Debug)]
pub(crate) struct ConnectedClient {
    user_id: String,
    server_info: ServerInfo,
    on_in_entities: Option<Cb<dyn Fn(&WorldDiff) + Send + Sync>>,
}

#[derive(Debug)]
pub(crate) enum ClientState {
    Connecting(String),
    Connected(ConnectedClient),
    Disconnected,
}

pub type SharedClientState = Arc<Mutex<ClientGameState>>;

impl ClientState {
    pub fn process_disconnect(&mut self) {
        tracing::info!("Disconnecting client: {self:#?}");

        *self = Self::Disconnected;
    }

    /// Processes an incoming control frame from the server.
    #[tracing::instrument(level = "info")]
    pub fn process_control(&mut self, state: &SharedClientState, frame: ClientControl) -> anyhow::Result<()> {
        match (frame, &self) {
            (ClientControl::ServerInfo(server_info), Self::Connecting(user_id)) => {
                tracing::info!("Received server info: {server_info:?}");

                let state = state.lock();
                ContentBaseUrlKey.insert(&state.assets, server_info.content_base_url.clone());

                *self = Self::Connected(ConnectedClient { user_id: user_id.clone(), server_info, on_in_entities: None });

                Ok(())
            }
            (ClientControl::ServerInfo(_), _) => {
                tracing::warn!("Received server info while already connected");
                Ok(())
            }
            (ClientControl::Disconnect, _) => {
                self.process_disconnect();
                Ok(())
            }
        }
    }
}

impl ConnectedClient {
    #[tracing::instrument(level = "info")]
    pub fn process_diff(&mut self, state: &SharedClientState, diff: WorldDiff) -> anyhow::Result<()> {
        // if let Some(on_in_entities) = &self.on_in_entities {
        //     on_in_entities(&diff);
        // }
        let mut gs = state.lock();
        diff.apply(&mut gs.world, Entity::new().with(is_remote_entity(), ()), false);
        Ok(())
    }

    /// Processes a server initiated bidirectional stream
    #[tracing::instrument(level = "info", skip(send, recv))]
    pub async fn process_bi<R, S>(&mut self, state: &SharedClientState, send: S, mut recv: R) -> anyhow::Result<()>
    where
        R: 'static + Send + Sync + Unpin + AsyncRead,
        S: 'static + Send + Sync + Unpin + AsyncWrite,
    {
        let id = recv.read_u32().await?;

        let mut gs = state.lock();
        let gs = &mut *gs;
        let world = &mut gs.world;
        let assets = gs.assets.clone();

        let handler = world.resource(bi_stream_handlers()).get(&id).with_context(|| format!("No handler for stream {id}"))?.clone();

        let _span = info_span!("handle_bi", id).entered();
        handler(world, assets, Box::pin(send), Box::pin(recv));

        Ok(())
    }

    /// Processes a server initiated unidirectional stream
    #[tracing::instrument(level = "info", skip(recv))]
    pub async fn process_uni<R>(&mut self, state: &SharedClientState, mut recv: R) -> anyhow::Result<()>
    where
        R: 'static + Send + Sync + Unpin + AsyncRead,
    {
        let id = recv.read_u32().await?;

        let mut gs = state.lock();
        let gs = &mut *gs;
        let world = &mut gs.world;
        let assets = gs.assets.clone();

        let handler = world.resource(uni_stream_handlers()).get(&id).with_context(|| format!("No handler for stream {id}"))?.clone();

        let _span = info_span!("handle_uni", id).entered();
        handler(world, assets, Box::pin(recv));

        Ok(())
    }

    /// Processes an incoming datagram
    #[tracing::instrument(level = "info")]
    pub fn process_datagram(&mut self, state: &SharedClientState, mut data: Bytes) -> anyhow::Result<()> {
        if data.len() < 4 {
            bail!("Received malformed datagram");
        }

        let id = data.get_u32();

        let mut gs = state.lock();
        let gs = &mut *gs;
        let world = &mut gs.world;
        let assets = gs.assets.clone();

        let handler = world.resource(datagram_handlers()).get(&id).with_context(|| format!("No handler for stream {id}"))?.clone();

        let _span = info_span!("handle_uni", id).entered();
        handler(world, assets, data);

        Ok(())
    }
}
