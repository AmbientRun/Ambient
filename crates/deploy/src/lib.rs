pub mod deploy_proto {
    include!("../proto/ambient.run.deploy.rs");
}

use std::{path::Path, str::FromStr};

use ambient_project::Manifest;
use tonic::{
    codegen::{CompressionEncoding, InterceptedService},
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig, Uri},
    Request,
};
use walkdir::WalkDir;

use deploy_proto::{deployer_client::DeployerClient, DeployAssetRequest, DeployAssetsResponse};

const CHUNK_SIZE: usize = 1024 * 1024 * 3;

async fn asset_requests_from_file_path(
    base_path: impl AsRef<Path>,
    file_path: impl AsRef<Path>,
) -> anyhow::Result<Vec<DeployAssetRequest>> {
    let path = file_path
        .as_ref()
        .strip_prefix(base_path)?
        .to_string_lossy()
        .to_string();
    let content = ambient_sys::fs::read(file_path.as_ref()).await?;
    let total_size = content.len() as u64;

    Ok(content
        .chunks(CHUNK_SIZE)
        .map(|chunk| DeployAssetRequest {
            path: path.clone(),
            total_size,
            content: chunk.to_vec(),
        })
        .collect())
}

/// This takes the path to an Ambient project and deploys it. An Ambient project is expected to
/// be already built.
pub async fn deploy(
    runtime: &tokio::runtime::Runtime,
    api_server: String,
    auth_token: &str,
    path: impl AsRef<Path>,
    manifest: &Manifest,
) -> anyhow::Result<DeployAssetsResponse> {
    log::info!(
        "Deploying project `{}` ({})",
        manifest.project.id,
        manifest
            .project
            .name
            .as_deref()
            .unwrap_or_else(|| manifest.project.id.as_ref())
    );

    // set up TLS config if needed
    let tls = if api_server.starts_with("https://") {
        let domain_name = Uri::from_str(&api_server)?
            .host()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not parse domain name from API server URI: {}",
                    api_server
                )
            })?
            .to_string();

        let mut tls = ClientTlsConfig::new().domain_name(domain_name);

        // set test CA cert if provided
        if let Ok(test_ca_cert) = std::env::var("AMBIENT_DEPLOY_TEST_CA_CERT") {
            let pem = std::fs::read_to_string(test_ca_cert)?;
            let ca = Certificate::from_pem(pem);
            tls = tls.ca_certificate(ca);
        }
        Some(tls)
    } else {
        None
    };

    // set up the endpoint and connect
    let channel = {
        let mut endpoint = Channel::from_shared(api_server)?;
        if let Some(tls) = tls {
            endpoint = endpoint.tls_config(tls)?
        }
        endpoint.connect().await?
    };

    // set up client with auth token and compression
    let token: MetadataValue<_> = format!("Bearer {}", auth_token).parse()?;
    let mut client = DeployerClient::new(InterceptedService::new(channel, {
        let token = token.clone();
        move |mut req: Request<()>| {
            req.metadata_mut().insert("authorization", token.clone());
            Ok(req)
        }
    }))
    .send_compressed(CompressionEncoding::Gzip)
    .accept_compressed(CompressionEncoding::Gzip);

    // iterate over all files to deploy (everything in the build directory)
    let file_paths = WalkDir::new(path.as_ref().join("build"))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|x| x.is_file()).unwrap_or(false))
        .map(|x| x.into_path());

    // create a separate task for reading files
    let (tx, rx) = flume::bounded::<DeployAssetRequest>(16);
    let base_path = path.as_ref().to_owned();
    let handle = runtime.spawn(async move {
        for file_path in file_paths {
            let requests = asset_requests_from_file_path(&base_path, file_path).await?;
            let count = requests.len();
            for (idx, request) in requests.into_iter().enumerate() {
                log::debug!(
                    "Deploying asset {} {}B/{}B ({}/{})",
                    request.path,
                    request.content.len(),
                    request.total_size,
                    idx + 1,
                    count
                );
                tx.send_async(request).await?;
            }
        }
        anyhow::Ok(())
    });

    let response = client.deploy_assets(rx.into_stream()).await?;

    // wait for the file reading task to finish to handle any errors
    handle.await??;

    Ok(response.into_inner())
}
