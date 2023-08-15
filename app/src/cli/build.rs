use ambient_native_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};
use anyhow::Context;

use crate::shared;

use super::{ProjectCli, ProjectPath};

pub async fn build(
    project: Option<&ProjectCli>,
    project_path: ProjectPath,
    assets: &AssetCache,
    release_build: bool,
) -> anyhow::Result<(ProjectPath, Option<AbsAssetUrl>)> {
    let Some(project) = project else {
        return Ok((project_path, None));
    };

    if project.no_build {
        return Ok((project_path, None));
    }

    let Some(project_path) = project_path.fs_path else {
        return Ok((project_path, None));
    };

    let build_path = project_path.join("build");
    // The build step uses its own semantic to ensure that there is
    // no contamination, so that the built project can use its own
    // semantic based on the flat hierarchy.
    let mut semantic = ambient_project_semantic::Semantic::new().await?;
    let primary_ember_scope_id = shared::ember::add(None, &mut semantic, &project_path).await?;

    let manifest = semantic
        .items
        .get(primary_ember_scope_id)?
        .manifest
        .clone()
        .context("no manifest for scope")?;

    let build_config = ambient_build::BuildConfiguration {
        build_path: build_path.clone(),
        assets: assets.clone(),
        semantic: &mut semantic,
        optimize: release_build,
        clean_build: project.clean_build,
        build_wasm_only: project.build_wasm_only,
    };

    let project_name = manifest
        .ember
        .name
        .as_deref()
        .unwrap_or_else(|| manifest.ember.id.as_str());

    tracing::info!("Building project {:?}", project_name);

    let output_path = ambient_build::build(build_config, primary_ember_scope_id)
        .await
        .context("Failed to build project")?;

    anyhow::Ok((
        ProjectPath::new_local(output_path)?,
        Some(AbsAssetUrl::from_file_path(build_path)),
    ))
}
