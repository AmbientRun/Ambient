use std::process::Stdio;

use anyhow::Context;
use futures::FutureExt;
use kiwi_std::asset_url::AssetType;
use kiwi_world_audio::AudioNode;
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::{info_span, Instrument};

use super::{
    context::PipelineCtx, out_asset::{asset_id_from_url, OutAsset, OutAssetContent, OutAssetPreview}
};

pub const SOUND_GRAPH_EXTENSION: &str = "sgr";

pub async fn pipeline(ctx: &PipelineCtx) -> Vec<OutAsset> {
    ctx.process_files(
        |file| matches!(file.extension().as_deref(), Some("ogg") | Some("wav") | Some("mp3")),
        |ctx, file| async move {
            let contents = file.download_bytes(ctx.assets()).await?;

            let filename = file.path().file_name().unwrap().to_string();

            let rel_path = ctx.in_root().relative_path(file.path());

            let content_url = match file.extension().as_deref() {
                Some("ogg") => ctx.write_file(&rel_path, contents).await,
                ext @ Some("wav" | "mp3") => {
                    tracing::info!("Processing {ext:?} file");
                    // Make sure to take the contents, to avoid having both the input and output in
                    // memory at once
                    let contents = ffmpeg_convert(std::io::Cursor::new(contents)).await?;
                    ctx.write_file(rel_path.with_extension("ogg"), contents).await
                }
                other => anyhow::bail!("Audio filetype {:?} is not yet supported", other.unwrap_or_default()),
            };

            let root_node = AudioNode::Vorbis { url: content_url.to_string() };
            let graph_url = ctx.write_file(&rel_path.with_extension("SOUND_GRAPH_EXTENSION"), save_audio_graph(root_node).unwrap()).await;

            Ok(vec![
                OutAsset {
                    id: asset_id_from_url(&file),
                    type_: AssetType::VorbisTrack,
                    hidden: false,
                    name: filename.clone(),
                    tags: Vec::new(),
                    categories: Default::default(),
                    preview: OutAssetPreview::None,
                    content: OutAssetContent::Content(content_url),
                    source: Some(file.clone()),
                },
                OutAsset {
                    id: asset_id_from_url(&file.push("graph").unwrap()),
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
