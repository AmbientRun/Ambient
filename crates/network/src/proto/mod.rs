use std::sync::Arc;

use ambient_std::asset_cache::AssetCache;
use anyhow::Context;
use bytes::{Buf, Bytes, BytesMut};
use flume::r#async::RecvStream;
use futures::{Stream, StreamExt};
use parking_lot::Mutex;
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::info_span;

use crate::server::{datagram_handlers, uni_stream_handlers, ServerState};

/// Handles the server side protocol logic.
pub struct Server {
    user_id: String,
    diffs_rx: RecvStream<'static, Bytes>,
    state: Arc<Mutex<ServerState>>,
    assets: AssetCache,
}

impl Server {
    pub fn new(user_id: String, diffs_rx: RecvStream<'static, Bytes>, state: Arc<Mutex<ServerState>>, assets: AssetCache) -> Self {
        Self { user_id, diffs_rx, state, assets }
    }

    /// Processes diffs, returning a stream of bytes to be sent over the network
    async fn process_diffs(&mut self) -> Option<Bytes> {
        self.diffs_rx.next().await
    }

    #[tracing::instrument(level = "info", skip(self))]
    /// Processes an incoming datagram
    async fn process_datagram(&mut self, mut datagram: Bytes) -> anyhow::Result<()> {
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
    async fn process_uni<E: Into<anyhow::Error>>(&mut self, mut stream: impl AsyncRead + Unpin) -> anyhow::Result<()> {
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
