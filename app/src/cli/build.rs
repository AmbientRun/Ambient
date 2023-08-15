use ambient_native_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};
use anyhow::Context;

use crate::shared;

use super::{ProjectCli, ProjectPath};

pub struct BuildDirectories {
    /// The location where all built embers are stored. Used for the HTTP host.
    pub build_root_path: AbsAssetUrl,
    /// The location of the main ember being executed. Used for everything else.
    pub main_ember_path: AbsAssetUrl,
}
impl BuildDirectories {
    pub fn new_with_same_paths(path: AbsAssetUrl) -> Self {
        Self {
            build_root_path: path.clone(),
            main_ember_path: path,
        }
    }
}

pub async fn build(
    project: Option<&ProjectCli>,
    project_path: ProjectPath,
    assets: &AssetCache,
    release_build: bool,
) -> anyhow::Result<BuildDirectories> {
    let Some(project) = project else {
        return Ok(BuildDirectories::new_with_same_paths(project_path.url.clone()));
    };

    if project.no_build {
        return Ok(BuildDirectories::new_with_same_paths(
            project_path.url.clone(),
        ));
    }

    let Some(project_path) = project_path.fs_path else {
        return Ok(BuildDirectories::new_with_same_paths(project_path.url.clone()));
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

    anyhow::Ok(BuildDirectories {
        build_root_path: AbsAssetUrl::from_file_path(build_path),
        main_ember_path: AbsAssetUrl::from_file_path(output_path),
    })
}
