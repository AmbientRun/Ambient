use ambient_ecs::WorldStreamFilter;
use ambient_std::asset_url::AbsAssetUrl;
use anyhow::Context;
use bytes::Bytes;
use h3_webtransport::server::WebTransportSession;
use http::Method;

use crate::server::SharedServerState;

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
    session: WebTransportSession<h3_quinn::Connection, Bytes>,
    state: SharedServerState,
    world_stream_filter: WorldStreamFilter,
    content_base_url: AbsAssetUrl,
) -> anyhow::Result<()> {
    loop {
        let datagram = session.accept_datagram().await?;
        tracing::info!("Received datagram: {datagram:?}");
    }
}
