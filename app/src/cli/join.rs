use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    download_asset::ReqwestClientKey,
};
use ambient_network::native::client::ResolvedAddr;
use clap::Parser;

use crate::{client, server::QUIC_INTERFACE_PORT};

use super::ClientCli;

#[derive(Parser, Clone, Debug)]
/// Join a multiplayer session
pub struct Join {
    #[command(flatten)]
    pub client: ClientCli,
    /// The server to connect to; defaults to localhost
    pub host: Option<String>,
}

pub fn handle(args: &Join, rt: &tokio::runtime::Runtime, assets: AssetCache) -> anyhow::Result<()> {
    let assets_ref = &assets;
    let server_addr = rt.block_on(async move {
        let Some(mut host) = args.host.as_ref().cloned() else {
            return Ok(ResolvedAddr::localhost_with_port(QUIC_INTERFACE_PORT));
        };

        if host.starts_with("http://") || host.starts_with("https://") {
            tracing::info!("NOTE: Joining server by http url is still experimental and can be removed without warning.");

            let reqwest = &ReqwestClientKey.get(assets_ref);
            host = reqwest.get(host).send().await?.text().await?;

            if host.is_empty() {
                anyhow::bail!("Failed to resolve host");
            }
        }
        if !host.contains(':') {
            host = format!("{host}:{QUIC_INTERFACE_PORT}");
        }
        ResolvedAddr::lookup_host(&host).await
    })?;
    client::run(rt, assets, server_addr, &args.client, None)
}
