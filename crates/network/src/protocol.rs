use ambient_ecs::{ComponentRegistry, ExternalComponentDesc, WorldDiff};
use ambient_std::asset_url::AbsAssetUrl;
use anyhow::{Context, Result};
use futures::io::BufReader;
use quinn::{Connection, RecvStream, SendStream};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{
    client_connection::ConnectionInner, codec::FramedCodec, next_bincode_bi_stream, open_bincode_bi_stream, proto::ServerRequest,
    IncomingStream, NetworkError, OutgoingStream,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct ClientProtocol {
    pub(crate) conn: Connection,
    pub(crate) stat_stream: IncomingStream<quinn::RecvStream>,
    client_info: ClientInfo,
    pub(crate) diff_stream: IncomingStream<quinn::RecvStream>,
    /// Miscellaneous info from the server
    pub(crate) server_info: ServerInfo,
}

impl ClientProtocol {
    pub async fn new(conn: Connection, player_id: String) -> Result<Self> {
        // Say who we are
        // The server will respond appropriately and return things such as
        // username (TODO)
        let (mut tx, mut rx) = open_bincode_bi_stream(&conn).await?;
        tx.send(&player_id).await?;

        // The server will acknowledge and send the credentials back
        let client_info: ClientInfo = rx.next().await?;
        ComponentRegistry::get_mut().add_external(client_info.external_components.clone());

        let server_info: ServerInfo = rx.next().await?;
        if server_info.version != VERSION {
            anyhow::bail!("Server version mismatch: expected {}, got {}", VERSION, server_info.version);
        }

        // Great, the server knows who we are.
        // Two streams are opened
        let mut diff_stream = IncomingStream::accept_incoming(&conn).await?;
        diff_stream.next().await?;

        let mut stat_stream = IncomingStream::accept_incoming(&conn).await?;
        stat_stream.next().await?;

        log::debug!("Setup client side protocol");

        Ok(Self { conn, diff_stream, stat_stream, client_info, server_info })
    }

    pub async fn next_diff(&mut self) -> anyhow::Result<WorldDiff> {
        self.diff_stream.next::<WorldDiff>().await.context("Failed to read world diff")
    }

    pub async fn next_event(&mut self) -> anyhow::Result<BufReader<RecvStream>> {
        let stream = self.conn.accept_uni().await.map_err(NetworkError::from).context("Event stream closed")?;

        let stream = BufReader::new(stream);
        Ok(stream)
    }

    pub fn client_info(&self) -> &ClientInfo {
        &self.client_info
    }

    pub(crate) fn connection(&self) -> quinn::Connection {
        self.conn.clone()
    }
}

type FramedRecvStream<T> = FramedRead<RecvStream, FramedCodec<T>>;
type FramedSendStream<T> = FramedWrite<SendStream, FramedCodec<T>>;

/// The server side connection to a client or a proxied client
pub struct ServerConnection {
    pub(crate) conn: ConnectionInner,
    pub(crate) control_send: FramedSendStream<ServerRequest>,
    pub(crate) control_recv: FramedRecvStream<ServerRequest>,

    pub(crate) diff_stream: OutgoingStream<quinn::RecvStream>,
    pub(crate) stat_stream: OutgoingStream<quinn::SendStream>,
    client_info: ClientInfo,
}

impl ServerConnection {
    /// Establishes a connection to the client
    pub async fn new(conn: ConnectionInner, server_info: ServerInfo) -> Result<Self, NetworkError> {
        // The client now opens a control stream
        let (mut send, mut recv) = conn.accept_bi().await?;
        let control_recv = FramedRead::new(recv, FramedCodec::new());
        let control_send = FramedWrite::new(send, FramedCodec::new());

        // let user_id: String = rx.next().await?;

        // log::debug!("Received handshake from {user_id:?}");

        // let external_components = ComponentRegistry::get().all_external().map(|x| x.0).collect();

        // // Respond
        // let client_info = ClientInfo { user_id, external_components };
        // log::debug!("Responding with {client_info:?}");
        // tx.send(&client_info).await?;

        // // Send the project name to the client so it can title its window correctly
        // tx.send(&server_info).await?;

        // Great, now open all required streams
        let mut diff_stream = OutgoingStream::new(conn.open_uni().await?);
        // // Send "something" to notify the client of the new stream
        // diff_stream.send(&()).await?;
        let mut stat_stream = OutgoingStream::new(conn.open_uni().await?);
        // stat_stream.send(&()).await?;

        // Ok(Self { conn, diff_stream, stat_stream, client_info })
        Ok(Self { conn, control_send, control_recv, diff_stream: todo!(), stat_stream: todo!(), client_info: todo!() })
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

/// Miscellaneous information about the server that needs to be sent to the client during the handshake.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerInfo {
    /// The name of the project. Used by the client to figure out what to title its window. Defaults to "Ambient".
    pub project_name: String,

    // Base url of the content server.
    pub content_base_url: AbsAssetUrl,

    /// The version of the server. Used by the client to determine whether or not to keep connecting.
    /// Defaults to the version of the crate.
    pub version: String,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            project_name: "Ambient".into(),
            content_base_url: AbsAssetUrl::parse("http://localhost:8999/content/").unwrap(),
            version: VERSION.into(),
        }
    }
}
