use ambient_native_std::asset_cache::AssetCache;
use anyhow::Context;
use clap::Parser;

use crate::{
    server::{self, ServerHandle},
    shared::certs::{CERT, CERT_KEY},
};

use super::{
    build::{self, BuildDirectories},
    HostCli, PackageArgs,
};

#[derive(Parser, Clone, Debug)]
/// Builds and runs the package in server-only mode
pub struct Serve {
    #[command(flatten)]
    pub package: PackageArgs,

    #[command(flatten)]
    pub host: HostCli,
}

pub async fn handle(
    serve: &Serve,
    assets: AssetCache,
    release_build: bool,
) -> anyhow::Result<ServerHandle> {
    handle_inner(&serve.package, &serve.host, assets, release_build).await
}

pub async fn handle_inner(
    package: &PackageArgs,
    host: &HostCli,
    assets: AssetCache,
    release_build: bool,
) -> anyhow::Result<ServerHandle> {
    let BuildDirectories {
        build_root_path,
        main_package_path,
        main_package_name: _,
    } = build::handle_inner(package, &assets, release_build).await?;

    let manifest = match main_package_path
        .push("ambient.toml")?
        .download_string(&assets)
        .await
    {
        Ok(toml) => ambient_package::Manifest::parse(&toml)?,
        Err(_) => {
            anyhow::bail!("Failed to find ambient.toml in package");
        }
    };
    let crypto = get_crypto(host)?;

    let working_directory = main_package_path
        .to_file_path()?
        .unwrap_or(std::env::current_dir()?);

    let server_handle = server::start(
        assets,
        host,
        build_root_path,
        main_package_path,
        working_directory,
        manifest,
        crypto,
    )
    .await;

    Ok(server_handle)
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
