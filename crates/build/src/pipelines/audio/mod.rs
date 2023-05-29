use ambient_std::asset_url::AssetType;
use ambient_world_audio::AudioNode;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::{info_span, Instrument};

use super::{
    context::PipelineCtx,
    out_asset::{asset_id_from_url, OutAsset, OutAssetContent, OutAssetPreview},
};

pub const SOUND_GRAPH_EXTENSION: &str = "sgr";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPipeline {
    /// Whether or not the audio should be converted to Ogg Vorbis.
    #[serde(default)]
    pub convert: bool,
}

pub async fn pipeline(ctx: &PipelineCtx, config: AudioPipeline) -> Vec<OutAsset> {
    ctx.process_files(
        |file| {
            matches!(
                file.extension().as_deref(),
                Some("ogg") | Some("wav") | Some("mp3")
            )
        },
        move |ctx, file| async move {
            let contents = file.download_bytes(ctx.assets()).await?;

            let filename = file.path().file_name().unwrap().to_string();

            let rel_path = ctx.in_root().relative_path(file.path());

            let content_url = match file.extension().as_deref() {
                Some("wav") => {
                    if config.convert {
                        tracing::info!("Processing wav file");
                        let contents = symphonia_convert("wav", contents).await?;
                        ctx.write_file(rel_path.with_extension("ogg"), contents)
                            .await
                    } else {
                        ctx.write_file(&rel_path, contents).await
                    }
                }
                Some("ogg") => ctx.write_file(&rel_path, contents).await,
                Some(ext @ "mp3") => {
                    tracing::info!("Processing mp3 file");
                    // Make sure to take the contents, to avoid having both the input and output in
                    // memory at once
                    let contents = symphonia_convert(ext, contents).await?;
                    ctx.write_file(rel_path.with_extension("ogg"), contents)
                        .await
                }
                other => anyhow::bail!(
                    "Audio filetype {:?} is not yet supported",
                    other.unwrap_or_default()
                ),
            };

            let root_node = AudioNode::Vorbis {
                url: content_url.to_string(),
            };
            let graph_url = ctx
                .write_file(
                    &rel_path.with_extension(SOUND_GRAPH_EXTENSION),
                    save_audio_graph(root_node).unwrap(),
                )
                .await;

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
    Ok(serde_json::to_string_pretty(&root)
        .context("Invalid sound graph")?
        .into_bytes())
}

#[tracing::instrument(level = "info", skip(input))]
async fn symphonia_convert(ext: &str, input: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    use std::num::{NonZeroU32, NonZeroU8};

    use symphonia::core::{
        codecs::{DecoderOptions, CODEC_TYPE_NULL},
        errors::Error,
        formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
    };

    use vorbis_rs::{VorbisBitrateManagementStrategy, VorbisEncoder};

    // this symphonia decoding code is largely based on symphonia's examples:
    // https://github.com/pdeljanov/Symphonia/blob/master/symphonia/examples

    // hint symphonia about what format this file might be
    let mut hint = Hint::new();
    hint.with_extension(ext);

    // create a media source stream with default options
    let media_source = Box::new(std::io::Cursor::new(input));
    let mss = MediaSourceStream::new(media_source, Default::default());

    // use default metadata and format reader options
    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();

    // probe the audio file for its params
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .context("Failed to probe audio format")?;
    let mut format = probed.format;

    // find the default audio track for this file
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .context("Failed to select default audio track")?;

    // init an audio decoder with default options
    let dec_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .context("Failed to create audio decoder")?;

    // randomize an ogg stream serial number
    let stream_serial: i32 = rand::random();

    // retrieve the sampling rate from the input file
    let sampling_rate: NonZeroU32 = decoder
        .codec_params()
        .sample_rate
        .context("Expected audio to have sample rate")?
        .try_into()
        .context("Audio must have >0 sampling rate")?;

    // retrieve the channel count from the input file
    let channels = decoder
        .codec_params()
        .channels
        .context("Audio does not have any channels")?
        .count();
    let channels: NonZeroU8 = (channels as u8)
        .try_into()
        .context("Audio must have >0 channels")?;

    // select a bitrate
    let bitrate = VorbisBitrateManagementStrategy::QualityVbr {
        target_quality: 0.9,
    };

    // create the ogg Vorbis encoder
    let mut encoder = VorbisEncoder::new(
        stream_serial,
        [("", ""); 0], // no tags
        sampling_rate,
        channels,
        bitrate,
        None,
        Vec::new(),
    )?;

    // process all packets in the input file
    let result = loop {
        // read the next packet
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(err) => break err,
        };

        // decode the packet's samples
        let decoded = match decoder.decode(&packet) {
            Ok(buf) => buf,
            Err(err) => break err,
        };

        // convert the decoded samples to f32 samples
        let mut block = decoded.make_equivalent::<f32>();
        decoded.convert(&mut block);

        // get the samples as &[&[f32]]
        let planes = block.planes();
        let plane_samples = planes.planes();

        // feed the samples into the encoder
        encoder.encode_audio_block(plane_samples)?;
    };

    // process the error returned by the loop
    match result {
        // "end of stream" is non-fatal, ignore it
        Error::IoError(err)
            if err.kind() == std::io::ErrorKind::UnexpectedEof
                && err.to_string() == "end of stream" => {}
        // return every other kind of error
        err => return Err(err.into()),
    }

    // finish encoding
    let output = encoder.finish()?;
    tracing::info!("Encoded {} samples", output.len());
    Ok(output)
}
