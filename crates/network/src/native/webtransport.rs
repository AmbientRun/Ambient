use ambient_ecs::WorldStreamFilter;
use ambient_std::asset_url::AbsAssetUrl;
use bytes::Bytes;
use http::Method;

use crate::server::SharedServerState;

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
        .send_grease(true)
        .build(h3_quinn::Connection::new(conn))
        .await
        .unwrap();

    loop {
        let req = conn.accept().await?;

        if let Some((req, resp)) = req {
            match req.method() {
                &Method::CONNECT => {}
                method => {
                    tracing::info!(?method, "Received other HTTP/3 request")
                }
            }
        }
    }
}
