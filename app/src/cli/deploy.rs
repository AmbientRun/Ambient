use ambient_native_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};

use crate::retrieve_manifest;

#[allow(clippy::too_many_arguments)]
pub async fn handle(
    built_package_path: &AbsAssetUrl,
    assets: &AssetCache,
    token: &str,
    api_server: &str,
    force_upload: bool,
    ensure_running: bool,
    context: &str,
) -> Result<(), anyhow::Error> {
    let manifest = retrieve_manifest(built_package_path, assets).await?;

    let Some(package_fs_path) = built_package_path.to_file_path()? else {
        anyhow::bail!("Can only deploy a local package");
    };

    let deployment_id =
        ambient_deploy::deploy(api_server, token, package_fs_path, &manifest, force_upload).await?;

    log::info!(
        "Assets deployed successfully. Deployment id: {}. Deploy url: https://assets.ambient.run/{}",
        deployment_id,
        deployment_id,
    );

    if ensure_running {
        let spec = ambient_cloud_client::ServerSpec::new_with_deployment(deployment_id)
            .with_context(context.to_string());
        let server =
            ambient_cloud_client::ensure_server_running(assets, api_server, token.into(), spec)
                .await?;
        log::info!("Deployed package is running at {}", server.host);
    }
    Ok(())
}
