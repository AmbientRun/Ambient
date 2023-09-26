use std::path::{Path, PathBuf};

use ambient_native_std::asset_cache::AssetCache;
use anyhow::Context;
use clap::{Args, Subcommand};

use super::PackagePath;

#[derive(Subcommand, Clone, Debug)]
pub enum Assets {
    /// Migrate json pipelines to toml
    #[command(name = "migrate-pipelines-toml")]
    MigratePipelinesToml(MigrateOptions),
    /// Import new assets with interactive prompts
    #[command(name = "import")]
    Import(ImportOptions),
}

#[derive(Args, Clone, Debug)]
pub struct MigrateOptions {
    #[arg(default_value = "./assets")]
    /// The path to the assets folder
    pub path: PathBuf,
}

#[derive(Args, Clone, Debug)]
pub struct ImportOptions {
    #[arg()]
    /// The path to the assets you want to import
    pub path: PathBuf,
    #[arg(long)]
    /// Whether to convert audio files to OGG
    pub convert_audio: bool,
    /// Whether to generate a collider from the model
    #[arg(long)]
    pub collider_from_model: bool,
}

pub async fn handle(command: &Assets, assets: &AssetCache) -> anyhow::Result<()> {
    match command {
        Assets::MigratePipelinesToml(opt) => {
            migrate_pipelines_toml(opt).await?;
        }
        Assets::Import(opt) => import(opt, assets).await?,
    }

    Ok(())
}

async fn migrate_pipelines_toml(opt: &MigrateOptions) -> Result<(), anyhow::Error> {
    let path = PackagePath::new_local(opt.path.clone())?;
    ambient_build::migrate::toml::process(path.fs_path.unwrap())
        .await
        .context("Failed to migrate pipelines")?;
    Ok(())
}

async fn import(opt: &ImportOptions, assets: &AssetCache) -> anyhow::Result<()> {
    let Some(ext) = opt.path.extension() else {
        anyhow::bail!("Unknown file type");
    };

    if ext == "wav" || ext == "mp3" || ext == "ogg" {
        let convert = opt.convert_audio;
        ambient_build::pipelines::import_audio(opt.path.clone(), convert)
            .context("Failed to import audio")?;
    } else if ext == "fbx" || ext == "glb" || ext == "gltf" || ext == "obj" {
        let collider_from_model = opt.collider_from_model;
        ambient_build::pipelines::import_model(opt.path.clone(), collider_from_model)
            .context("Failed to import models")?;
    } else if ext == "jpg" || ext == "png" || ext == "gif" || ext == "webp" {
        // TODO: import textures API may change, so this is just a placeholder
        todo!();
    } else {
        anyhow::bail!("Unsupported file type");
    }
    ambient_build::build_assets(assets, Path::new("assets"), Path::new("build"), true).await?;

    Ok(())
}
