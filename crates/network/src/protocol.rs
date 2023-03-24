use ambient_ecs::{ComponentRegistry, ExternalComponentDesc, WorldDiff};
use anyhow::{Context, Result};
use futures::{io::BufReader, StreamExt};
use quinn::{NewConnection, RecvStream};

use crate::{next_bincode_bi_stream, open_bincode_bi_stream, server::ServerInfo, IncomingStream, NetworkError, OutgoingStream, client_connection::ClientConnection};

#[derive(Debug)]
pub struct ClientProtocol {
    pub(crate) conn: NewConnection,
    pub(crate) stat_stream: IncomingStream,
    client_info: ClientInfo,
    pub(crate) diff_stream: IncomingStream,
    /// Miscellaneous info from the server
    pub(crate) server_info: ServerInfo,
}

impl ClientProtocol {
    pub async fn new(mut conn: NewConnection, player_id: String) -> Result<Self> {
        // Say who we are
        // The server will respond appropriately and return things such as
        // username (TODO)
        let (mut tx, mut rx) = open_bincode_bi_stream(&conn.connection).await?;
        tx.send(&player_id).await?;

        // The server will acknowledge and send the credentials back
        let client_info: ClientInfo = rx.next().await?;
        ComponentRegistry::get_mut().add_external(client_info.external_components.clone());

        let server_info: ServerInfo = rx.next().await?;
        // Great, the server knows who we are.
        // Two streams are opened
        let mut diff_stream = IncomingStream::accept_incoming(&mut conn).await?;
        diff_stream.next().await?;

        let mut stat_stream = IncomingStream::accept_incoming(&mut conn).await?;
        stat_stream.next().await?;

        log::debug!("Setup client side protocol");

        Ok(Self { conn, diff_stream, stat_stream, client_info, server_info })
    }

    pub async fn next_diff(&mut self) -> anyhow::Result<WorldDiff> {
        self.diff_stream.next::<WorldDiff>().await.context("Failed to read world diff")
    }

    pub async fn next_event(&mut self) -> anyhow::Result<BufReader<RecvStream>> {
        let stream = self.conn.uni_streams.next().await.ok_or(NetworkError::EndOfStream).context("Event stream closed")??;

        let stream = BufReader::new(stream);
        Ok(stream)
    }

    pub fn client_info(&self) -> &ClientInfo {
        &self.client_info
    }

    pub(crate) fn connection(&self) -> quinn::Connection {
        self.conn.connection.clone()
    }

    pub fn uni_streams(&self) -> &quinn::IncomingUniStreams {
        &self.conn.uni_streams
    }
}

/// The server side protocol instantiation of the client communication
pub struct ServerProtocol {
    pub(crate) conn: ClientConnection,

    pub(crate) diff_stream: OutgoingStream,
    pub(crate) stat_stream: OutgoingStream,
    client_info: ClientInfo,
}

impl ServerProtocol {
    pub async fn new(mut conn: ClientConnection, server_info: ServerInfo) -> Result<Self, NetworkError> {
        // The client now sends the player id
        let (mut tx, mut rx) = conn.accept_bincode_bi().await?;

        let user_id: String = rx.next().await?;

        log::debug!("Received handshake from {user_id:?}");

        let external_components = ComponentRegistry::get().all_external().map(|x| x.0).collect();

        // Respond
        let client_info = ClientInfo { user_id, external_components };
        log::debug!("Responding with {client_info:?}");
        tx.send(&client_info).await?;

        // Send the project name to the client so it can title its window correctly
        tx.send(&server_info).await?;

        // Great, now open all required streams
        let mut diff_stream = conn.open_bincode_uni().await?;
        // Send "something" to notify the client of the new stream
        diff_stream.send(&()).await?;
        let mut stat_stream = conn.open_bincode_uni().await?;
        stat_stream.send(&()).await?;

        Ok(Self { conn, diff_stream, stat_stream, client_info })
    }

    pub fn client_info(&self) -> &ClientInfo {
        &self.client_info
    }
}

/// Contains things such as username (TODO) and user_id
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientInfo {
    pub user_id: String,
    pub external_components: Vec<ExternalComponentDesc>,
}

impl std::fmt::Debug for ClientInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientInfo").field("user_id", &self.user_id).finish_non_exhaustive()
    }
}
