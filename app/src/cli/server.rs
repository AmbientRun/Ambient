use std::path::PathBuf;

use ambient_native_std::asset_cache::AssetCache;
use ambient_network::native::client::ResolvedAddr;
use anyhow::Context;

use crate::{
    retrieve_manifest, server,
    shared::certs::{CERT, CERT_KEY},
};

use super::{build::BuildDirectories, HostCli};

pub async fn handle(
    host: &HostCli,
    view_asset_path: Option<PathBuf>,
    directories: BuildDirectories,
    assets: &AssetCache,
) -> anyhow::Result<ResolvedAddr> {
    let BuildDirectories {
        build_root_path,
        main_package_path,
    } = directories;

    let manifest = retrieve_manifest(&main_package_path, assets).await?;
    let crypto = get_crypto(host)?;

    let working_directory = main_package_path
        .to_file_path()?
        .clone()
        .unwrap_or(std::env::current_dir()?);

    let addr = server::start(
        assets.clone(),
        host,
        build_root_path,
        main_package_path,
        view_asset_path,
        working_directory,
        manifest,
        crypto,
    )
    .await;

    Ok(ResolvedAddr::localhost_with_port(addr.port()))
}

fn get_crypto(host: &HostCli) -> anyhow::Result<ambient_network::native::server::Crypto> {
    let Some((cert_file, key_file)) = host.cert.as_ref().zip(host.key.as_ref()) else {

        #[cfg(feature = "no_bundled_certs")]
        {
            anyhow::bail!("--cert and --key are required without bundled certs.");
        }
        #[cfg(not(feature = "no_bundled_certs"))]
        {
            tracing::info!("Using bundled certificate and key");
            return Ok(ambient_network::native::server::Crypto {
                            cert_chain: vec![CERT.to_vec()],
                            key: CERT_KEY.to_vec(),
                        });
        }
    };

    let raw_cert = std::fs::read(cert_file).context("Failed to read certificate file")?;
    let cert_chain = if raw_cert.starts_with(b"-----BEGIN CERTIFICATE-----") {
        rustls_pemfile::certs(&mut raw_cert.as_slice())
            .context("Failed to parse certificate file")?
    } else {
        vec![raw_cert]
    };
    let raw_key = std::fs::read(key_file).context("Failed to read certificate key")?;
    let key = if raw_key.starts_with(b"-----BEGIN ") {
        rustls_pemfile::read_all(&mut raw_key.as_slice())
            .context("Failed to parse certificate key")?
            .into_iter()
            .find_map(|item| match item {
                rustls_pemfile::Item::RSAKey(key) => Some(key),
                rustls_pemfile::Item::PKCS8Key(key) => Some(key),
                rustls_pemfile::Item::ECKey(key) => Some(key),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("No private key found"))?
    } else {
        raw_key
    };

    Ok(ambient_network::native::server::Crypto { cert_chain, key })
}
