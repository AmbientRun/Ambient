use std::path::{Path, PathBuf};

use ambient_project::Manifest;
use anyhow::Context;
use futures::{future::ready, stream, StreamExt, TryStreamExt};
use serde::Deserialize;

use crate::{
    get_asset_files,
    pipelines::{Pipeline, PipelineSchema},
    register_from_manifest,
};

pub async fn process(manifest: &Manifest, path: PathBuf) -> anyhow::Result<()> {
    register_from_manifest(&manifest);
    let assets_path = path.join("assets");

    stream::iter(get_asset_files(&assets_path))
        .filter(|path| ready(path.ends_with("pipeline.json")))
        .then(|path: PathBuf| async move {
            migrate_pipeline(&path)
                .await
                .with_context(|| format!("Error migrating pipeline {path:?}"))?;

            Ok(())
        })
        .try_collect::<()>()
        .await
}

async fn migrate_pipeline(path: &Path) -> anyhow::Result<()> {
    #[derive(Debug, Clone, Deserialize)]
    #[serde(untagged)]
    enum PipelineOneOrMany {
        Many(Vec<Pipeline>),
        One(Pipeline),
    }

    impl PipelineOneOrMany {
        fn into_vec(self) -> Vec<Pipeline> {
            match self {
                PipelineOneOrMany::Many(v) => v,
                PipelineOneOrMany::One(p) => vec![p],
            }
        }
    }

    tracing::info!(?path, "Processing pipeline");

    let str = tokio::fs::read_to_string(path)
        .await
        .context("Error reading json pipeline file")?;

    tracing::info!("Read string: {str}");

    let de = &mut serde_json::de::Deserializer::from_str(&str);

    let value: PipelineOneOrMany = serde_path_to_error::deserialize(de)
        .with_context(|| format!("Error deserializing json pipeline file {:?}", path))?;

    tracing::info!("Deserialized json pipeline file: {value:#?}");

    let value = PipelineSchema {
        pipelines: value.into_vec(),
    };

    let toml = toml::to_string_pretty(&value).context("Error serializing json to toml")?;

    tracing::info!("Serialized to toml: {toml}");

    let toml_path = path.with_extension("toml");
    tokio::fs::write(&toml_path, toml)
        .await
        .context("Error writing toml pipeline file")?;

    eprintln!("Wrote toml pipeline file: {:?}", toml_path);

    Ok(())
}
