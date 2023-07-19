use std::sync::Arc;

use ambient_ecs::WorldStreamFilter;
use ambient_std::asset_url::AbsAssetUrl;
use anyhow::Context;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use h3::quic::BidiStream;
use h3_webtransport::server::{AcceptedBi, WebTransportSession};
use http::Method;
use uuid::Uuid;

use crate::{
    proto::{
        self,
        server::{handle_diffs, ConnectionData},
        ServerInfo, ServerPush,
    },
    server::SharedServerState,
    stream::{FramedRecvStream, FramedSendStream},
    NetworkError,
};

#[tracing::instrument(level = "info", skip_all)]
pub async fn handle_h3_connection(
    conn: quinn::Connection,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    content_base_url: AbsAssetUrl,
) -> anyhow::Result<()> {
    // Establish an HTTP/3 connection
    //
    // The webtransport client will soon send a `CONNECT` request
    let mut conn: h3::server::Connection<h3_quinn::Connection, Bytes> = h3::server::builder()
        .enable_webtransport(true)
        .enable_connect(true)
        .enable_datagram(true)
        .max_webtransport_sessions(1)
        .build(h3_quinn::Connection::new(conn))
        .await
        .unwrap();

    loop {
        let req = conn.accept().await?;

        if let Some((req, resp)) = req {
            match req.method() {
                &Method::CONNECT => {
                    let session = WebTransportSession::accept(req, resp, conn)
                        .await
                        .context("Failed to accept webtransport session")?;
                    tracing::info!("Accepted webtransport session");

                    return handle_webtransport_session(
                        session,
                        state,
                        world_stream_filter,
                        content_base_url,
                    )
                    .await;
                }
                method => {
                    tracing::info!(?method, "Received other HTTP/3 request")
                }
            }
        }
    }
}

#[tracing::instrument(level = "info", skip_all)]
async fn handle_webtransport_session(
    conn: WebTransportSession<h3_quinn::Connection, Bytes>,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    content_base_url: AbsAssetUrl,
) -> anyhow::Result<()> {
    let conn = Arc::new(conn);

    let sid = conn.session_id();
    let (diffs_tx, diffs_rx) = flume::unbounded();

    let server_info = ServerInfo::new(&mut state.lock(), content_base_url);

    let mut server = proto::server::ServerProtoState::default();

    let mut request_recv = FramedRecvStream::new(
        conn.accept_uni()
            .await?
            .ok_or(NetworkError::ConnectionClosed)?
            .1,
    );

    let mut push_send = FramedSendStream::new(conn.open_uni(sid).await?);

    let diffs_rx = diffs_rx.into_stream();

    // Send who we are
    push_send.send(ServerPush::ServerInfo(server_info)).await?;

    // Feed the channel senders to the connection data
    //
    // Once connected they will be added to the player entity
    let data = ConnectionData {
        conn: conn.clone(),
        state,
        diff_tx: diffs_tx,
        connection_id: Uuid::new_v4(),
        world_stream_filter,
    };

    while server.is_pending_connection() {
        if let Some(frame) = request_recv.next().await {
            server.process_control(&data, frame?)?;
        }
    }

    tokio::spawn(handle_diffs(
        FramedSendStream::new(conn.open_uni(sid).await?),
        diffs_rx,
    ));

    let mut server = scopeguard::guard(server, |mut server| {
        if !server.is_disconnected() {
            tracing::info!("Connection closed abruptly from {server:?}");
            server.process_disconnect(&data);
        }
    });

    // Before a connection has been established, only process the control stream
    while let proto::server::ServerProtoState::Connected(connected) = &mut *server {
        tokio::select! {
            Some(frame) = request_recv.next() => {
                server.process_control(&data, frame?)?;
            }
            stream = conn.accept_uni() => {
                let (_, stream) = stream?.ok_or(NetworkError::ConnectionClosed)?;
                connected.process_uni(&data, stream);
            }
            stream = conn.accept_bi() => {
                if let AcceptedBi::BidiStream(_, stream) = stream?.ok_or(NetworkError::ConnectionClosed)? {
                    let (send, recv) = stream.split();

                    connected.process_bi(&data, send, recv);
                }
            }
            datagram = conn.accept_datagram() => {
                connected.process_datagram(&data, datagram?.ok_or(NetworkError::ConnectionClosed)?.1)?;
            }
            Some(msg) = connected.control_rx.next() => {
                push_send.send(&msg).await?;
            }
        }
    }

    tracing::info!("Client disconnected");

    Ok(())
}
