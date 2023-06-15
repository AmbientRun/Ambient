pub mod deploy_proto {
    include!("../proto/ambient.run.deploy.rs");
}

use std::{path::Path, str::FromStr};

use ambient_project::Manifest;
use tokio_stream::StreamExt;
use tonic::{
    codegen::{CompressionEncoding, InterceptedService},
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig, Uri},
    Request,
};
use walkdir::WalkDir;

use deploy_proto::{
    asset_content::ContentDescription, deploy_asset_response::Message,
    deployer_client::DeployerClient, AssetContent, DeployAssetRequest, VersionDeployed,
};

const CHUNK_SIZE: usize = 1024 * 1024 * 3; // 3MB

async fn asset_requests_from_file_path(
    ember_id: impl AsRef<str>,
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
            ember_id: ember_id.as_ref().into(),
            content: Some(AssetContent {
                path: path.clone(),
                total_size,
                content_description: Some(ContentDescription::Chunk(chunk.to_vec())),
            }),
        })
        .collect())
}

/// This takes the path to an Ambient ember and deploys it. An Ambient ember is expected to
/// be already built.
pub async fn deploy(
    runtime: &tokio::runtime::Runtime,
    api_server: String,
    auth_token: &str,
    path: impl AsRef<Path>,
    manifest: &Manifest,
) -> anyhow::Result<String> {
    let ember_id = manifest.ember.id.to_string();
    log::info!(
        "Deploying ember `{}` ({})",
        ember_id,
        manifest
            .ember
            .name
            .as_deref()
            .unwrap_or_else(|| manifest.ember.id.as_ref())
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
        for (file_idx, file_path) in file_paths.into_iter().enumerate() {
            let requests = asset_requests_from_file_path(&ember_id, &base_path, file_path).await?;
            let count = requests.len();
            for (idx, request) in requests.into_iter().enumerate() {
                let Some(content) = request.content.as_ref() else { unreachable!() };
                let Some(ContentDescription::Chunk(chunk)) = content.content_description.as_ref() else { unreachable!() };
                log::debug!(
                    "Deploying asset #{} chunk {}/{} {} {}B/{}B",
                    file_idx,
                    idx + 1,
                    count,
                    content.path,
                    chunk.len(),
                    content.total_size,
                );
                tx.send_async(request).await?;
            }
        }
        anyhow::Ok(())
    });

    let response = client.deploy_assets(rx.into_stream()).await?;
    let mut response_stream = response.into_inner();
    let mut version = None;
    while let Some(resp) = response_stream.next().await {
        match resp {
            Ok(resp) => {
                log::debug!("Deployed asset {:?}", resp);
                match resp.message {
                    Some(Message::Finished(VersionDeployed { id })) => {
                        if version.is_some() {
                            log::warn!("Received multiple version deployed messages");
                        }
                        version = Some(id);
                    }
                    Some(Message::Error(err)) => {
                        log::error!("Received error message: {:?}", err);
                        handle.abort();
                    }
                    Some(Message::MissingPath(path)) => {
                        // this shouldn't happen as we don't send MD5 hashes (used for differential uploads)
                        log::warn!("Received missing path message for asset: {:?}", path);
                    }
                    None => {
                        log::warn!("Received empty message");
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to deploy asset: {:?}", err);
            }
        }
    }

    // wait for the file reading task to finish to handle any errors
    handle.await??;

    version.ok_or_else(|| anyhow::anyhow!("No version returned from deploy"))
}
