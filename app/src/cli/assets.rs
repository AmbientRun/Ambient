use std::path::PathBuf;

use anyhow::Context;
use clap::{Args, Subcommand};

use super::ProjectPath;

#[derive(Subcommand, Clone, Debug)]
pub enum AssetCommand {
    /// Migrate json pipelines to toml
    #[command(name = "migrate-pipelines-toml")]
    MigratePipelinesToml(MigrateOptions),
    /// Import new assets with interactive prompts
    #[command(name = "import")]
    Import(ImportOptions),
}

#[derive(Args, Clone, Debug)]
pub struct MigrateOptions {
    #[arg(index = 1, default_value = "./assets")]
    /// The path to the assets folder
    pub path: PathBuf,
}

#[derive(Args, Clone, Debug)]
pub struct ImportOptions {
    #[arg(index = 1)]
    /// The path to the assets you want to import
    pub path: PathBuf,
    #[arg(long)]
    /// Whether to convert audio files to OGG
    pub convert_audio: bool,
    /// Whether to generate a collider from the model
    #[arg(long)]
    pub collider_from_model: bool,
}

pub async fn handle(command: &AssetCommand) -> anyhow::Result<()> {
    match command {
        AssetCommand::MigratePipelinesToml(opt) => {
            let path = ProjectPath::new_local(opt.path.clone())?;
            ambient_build::migrate::toml::process(path.fs_path.unwrap())
                .await
                .context("Failed to migrate pipelines")?;
        }
        AssetCommand::Import(opt) => match opt.path.extension() {
            Some(ext) => {
                if ext == "wav" || ext == "mp3" || ext == "ogg" {
                    let convert = opt.convert_audio;
                    ambient_build::pipelines::import_audio(opt.path.clone(), convert)
                        .context("failed to import audio")?;
                } else if ext == "fbx" || ext == "glb" || ext == "gltf" || ext == "obj" {
                    let collider_from_model = opt.collider_from_model;
                    ambient_build::pipelines::import_model(opt.path.clone(), collider_from_model)
                        .context("failed to import models")?;
                } else if ext == "jpg" || ext == "png" || ext == "gif" || ext == "webp" {
                    // TODO: import textures API may change, so this is just a placeholder
                    todo!();
                } else {
                    anyhow::bail!("Unsupported file type");
                }
            }
            None => anyhow::bail!("Unknown file type"),
        },
    }

    Ok(())
}
