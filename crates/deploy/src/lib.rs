pub mod deploy_proto {
    tonic::include_proto!("ambient.run.deploy");
}

use std::path::Path;

use ambient_project::Manifest;
use tonic::{
    codegen::{CompressionEncoding, InterceptedService},
    metadata::MetadataValue,
    transport::Channel,
    Request,
};
use walkdir::WalkDir;

use deploy_proto::{deployer_client::DeployerClient, DeployAssetRequest, DeployAssetsResponse};

async fn asset_from_file_path(
    base_path: impl AsRef<Path>,
    file_path: impl AsRef<Path>,
) -> anyhow::Result<DeployAssetRequest> {
    let path = file_path
        .as_ref()
        .strip_prefix(base_path)?
        .to_string_lossy()
        .to_string();
    let content = ambient_sys::fs::read(file_path.as_ref()).await?;
    Ok(DeployAssetRequest { path, content })
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

    let channel = Channel::from_shared(api_server)?.connect().await?;

    let token: MetadataValue<_> = format!("Bearer {}", auth_token).parse()?;

    let mut client = DeployerClient::new(InterceptedService::new(
        channel,
        move |mut req: Request<()>| {
            req.metadata_mut().insert("authorization", token.clone());
            Ok(req)
        },
    ))
    .send_compressed(CompressionEncoding::Gzip)
    .accept_compressed(CompressionEncoding::Gzip);

    // iterate over all files to deploy (everything in the build directory)
    let file_paths = WalkDir::new(path.as_ref().join("build"))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|x| x.is_file()).unwrap_or(false))
        .map(|x| x.into_path());

    // create a separate task for reading files
    let (tx, rx) = flume::unbounded::<DeployAssetRequest>();
    let base_path = path.as_ref().to_owned();
    let handle = runtime.spawn(async move {
        for file_path in file_paths {
            let asset = asset_from_file_path(&base_path, file_path).await?;
            log::debug!("Deploying asset {} {}b", asset.path, asset.content.len());
            tx.send(asset)?;
        }
        Ok(()) as anyhow::Result<()>
    });

    let response = client.deploy_assets(rx.into_stream()).await?;

    // wait for the file reading task to finish to handle any errors
    handle.await??;

    Ok(response.into_inner())
}
