use std::sync::Arc;

use ambient_ecs::{generated::components::core::network::is_remote_entity, Entity, WorldDiff};
use ambient_std::asset_cache::AssetCache;
use anyhow::Context;
use parking_lot::Mutex;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tracing::info_span;

use crate::{client::bi_stream_handlers, client_game_state::ClientGameState, proto::*, protocol::ServerInfo};

/// The client side of the protocol
pub struct Client {
    user_id: String,
    server_info: Option<ServerInfo>,
    connected: bool,
    game_state: Arc<Mutex<ClientGameState>>,
    on_in_entities: Option<Arc<dyn Fn(&WorldDiff) + Send + Sync>>,
    assets: AssetCache,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").field("server_info", &self.server_info).field("connected", &self.connected).finish_non_exhaustive()
    }
}

impl Client {
    /// Attempt to connect to the server.
    ///
    /// The client assumes a connection is established, if otherwise, the transport is closed.
    pub fn connect(&mut self) -> ServerControl {
        assert!(!self.connected);
        tracing::info!(user_id = self.user_id, "Sending connect request");

        self.connected = true;
        ServerControl::Connect(self.user_id.clone())
    }

    /// Processes an incoming control frame from the server.
    pub fn process_control(&mut self, frame: ClientControl) -> anyhow::Result<()> {
        match frame {
            ClientControl::ServerInfo(info) => {
                tracing::info!("Received server info");
                self.server_info = Some(info);
                Ok(())
            }
            ClientControl::Disconnect => {
                tracing::info!("Server disconnected");
                Ok(())
            }
        }
    }

    #[tracing::instrument(level = "info")]
    pub fn process_diff(&mut self, diff: WorldDiff) -> anyhow::Result<()> {
        if let Some(on_in_entities) = &self.on_in_entities {
            on_in_entities(&diff);
        }
        let mut gs = self.game_state.lock();
        diff.apply(&mut gs.world, Entity::new().with(is_remote_entity(), ()), false);
        Ok(())
    }

    /// Processes a server initiated bidirectional stream
    #[tracing::instrument(level = "info", skip(send, recv))]
    pub async fn process_bi_stream<R, S>(&mut self, send: S, mut recv: R) -> anyhow::Result<()>
    where
        R: 'static + Send + Sync + Unpin + AsyncRead,
        S: 'static + Send + Sync + Unpin + AsyncWrite,
    {
        let id = recv.read_u32().await?;
        let mut gs = self.game_state.lock();
        let world = &mut gs.world;

        let handler = world.resource(bi_stream_handlers()).get(&id).with_context(|| format!("No handler for stream {id}"))?.clone();

        let _span = info_span!("process_bi_stream", id).entered();
        handler(world, self.assets.clone(), Box::pin(send), Box::pin(recv));

        Ok(())
    }
}
