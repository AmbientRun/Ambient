use std::process::Stdio;

use anyhow::Context;
use elements_model_import::model_crate::ModelCrate;
use elements_std::asset_url::AssetType;
use elements_world_audio::AudioNode;
use futures::FutureExt;
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::{info_span, Instrument};

use super::{
    context::{AssetCrate, PipelineCtx}, out_asset::{OutAsset, OutAssetContent, OutAssetPreview}
};
use crate::helpers::download_bytes;

pub const SOUND_GRAPH_EXTENSION: &str = "sgr";

pub async fn pipeline(ctx: &PipelineCtx) -> Vec<OutAsset> {
    ctx.process_files(
        |file| {
            let ext = file.sub_path.extension.as_deref();

            matches!(ext, Some("ogg") | Some("wav") | Some("mp3"))
        },
        |ctx, file| async move {
            let id = ctx.asset_crate_id(&file.sub_path_string);
            let asset_crate = AssetCrate::new(&ctx, id.clone());
            let contents = download_bytes(&ctx.assets, &file.temp_download_url).await?;

            let ext = file.sub_path.extension.as_deref();
            let filename = file.sub_path.filename.to_string();

            let content_url = match ext {
                Some("ogg") => asset_crate.write_file(AssetType::VorbisTrack, &format!("{}.ogg", ModelCrate::MAIN), contents).await,
                Some("wav" | "mp3") => {
                    tracing::info!("Processing {ext:?} file");
                    // Make sure to take the contents, to avoid having both the input and output in
                    // memory at once
                    let contents = ffmpeg_convert(std::io::Cursor::new(contents)).await?;
                    asset_crate.write_file(AssetType::VorbisTrack, &format!("{}.ogg", ModelCrate::MAIN), contents).await
                }
                other => anyhow::bail!("Audio filetype {:?} is not yet supported", other.unwrap_or_default()),
            };

            let root_node = AudioNode::Vorbis { url: content_url.to_string() };
            let graph_url = asset_crate
                .write_file(
                    AssetType::SoundGraph,
                    &format!("{}.{SOUND_GRAPH_EXTENSION}", ModelCrate::MAIN),
                    save_audio_graph(root_node).unwrap(),
                )
                .await;

            Ok(vec![
                OutAsset {
                    asset_crate_id: id.clone(),
                    sub_asset: None,
                    type_: AssetType::VorbisTrack,
                    hidden: false,
                    name: filename.clone(),
                    tags: Vec::new(),
                    categories: Default::default(),
                    preview: OutAssetPreview::None,
                    content: OutAssetContent::Content(content_url),
                    source: Some(file.sub_path_string.clone()),
                },
                OutAsset {
                    asset_crate_id: id,
                    sub_asset: None,
                    type_: AssetType::SoundGraph,
                    hidden: false,
                    name: filename,
                    tags: Vec::new(),
                    categories: Default::default(),
                    preview: OutAssetPreview::None,
                    content: OutAssetContent::Content(graph_url),
                    source: None,
                },
            ])
        },
    )
    .instrument(info_span!("audio_pipeline"))
    .await
}

fn save_audio_graph(root: AudioNode) -> anyhow::Result<Vec<u8>> {
    Ok(serde_json::to_string_pretty(&root).context("Invalid sound graph")?.into_bytes())
}

#[tracing::instrument(level = "info", skip(input))]
async fn ffmpeg_convert<A>(input: A) -> anyhow::Result<Vec<u8>>
where
    A: 'static + Send + AsyncRead,
{
    let mut child = tokio::process::Command::new("ffmpeg")
        .args(["-i", "pipe:", "-f", "ogg", "pipe:1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to execute ffmpeg")?;

    tracing::info!("Writing to stdin");

    let mut stdin = child.stdin.take().expect("no stdin");
    let mut stdout = child.stdout.take().expect("no stdout");

    let input = tokio::task::spawn(async move {
        tokio::pin!(input);
        tokio::io::copy(&mut input, &mut stdin).await.context("Failed to write to stdin")
    })
    .map(|v| v.unwrap());

    let output = async move {
        let mut output = Vec::new();
        stdout.read_to_end(&mut output).await.unwrap();
        Ok(output)
    };

    let status = async { child.wait().await.context("Failed to wait for ffmpeg") };

    tracing::info!("Waiting for ffmpeg to complete");
    let (_, output, status) = tokio::try_join!(input, output, status)?;

    if !status.success() {
        anyhow::bail!("FFMPEG conversion failed")
    }

    tracing::info!("Converted to vorbis of {} kb", output.len() as f32 / 1000.0);

    Ok(output)
}
