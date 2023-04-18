use ambient_audio::AudioFromUrl; //  track::TrackDecodeStream, Source
use ambient_core::{asset_cache, async_ecs::async_run, runtime};
use ambient_ecs::World;
use ambient_std::asset_cache::AsyncAssetKeyExt;
use ambient_std::asset_url::AbsAssetUrl; // AssetUrl
use ambient_world_audio::{audio_sender, AudioMessage}; // audio_tracks,
use anyhow::Context;

pub(crate) fn load(world: &World, url: String) -> anyhow::Result<()> {
    let assets = world.resource(asset_cache()).clone();
    let asset_url = AbsAssetUrl::from_asset_key(url).to_string();
    let audio_url = AudioFromUrl {
        url: AbsAssetUrl::parse(asset_url).context("Failed to parse audio url")?,
    };
    let _track = audio_url.peek(&assets);
    Ok(())
}

pub(crate) fn play(
    world: &World,
    url: String,
    looping: bool,
    amp: f32,
    uid: u32,
) -> anyhow::Result<()> {
    let assets = world.resource(asset_cache()).clone();
    let asset_url = AbsAssetUrl::from_asset_key(url).to_string();
    let audio_url = AudioFromUrl {
        url: AbsAssetUrl::parse(asset_url.clone()).context("Failed to parse audio url")?,
    };
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    runtime.spawn(async move {
        let track = audio_url.get(&assets).await;
        async_run.run(move |world| {
            match track {
                Ok(track) => {
                    let sender = world.resource(audio_sender());
                    sender
                        .send(AudioMessage::Track(
                            track,
                            looping,
                            amp,
                            asset_url.replace("ambient-assets:/", ""),
                            uid,
                        ))
                        .unwrap();
                }
                Err(e) => log::error!("{e:?}"),
            };
        });
    });
    Ok(())
}

pub(crate) fn stop(world: &World, url: String) -> anyhow::Result<()> {
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    runtime.spawn(async move {
        async_run.run(move |world| {
            let sender = world.resource(audio_sender());
            sender.send(AudioMessage::Stop(url)).unwrap();
        });
    });
    Ok(())
}

pub(crate) fn set_amp(world: &World, url: String, amp: f32) -> anyhow::Result<()> {
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    runtime.spawn(async move {
        async_run.run(move |world| {
            let sender = world.resource(audio_sender());
            sender.send(AudioMessage::UpdateVolume(url, amp)).unwrap();
        });
    });
    Ok(())
}

pub(crate) fn stop_by_id(world: &World, uid: u32) -> anyhow::Result<()> {
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    runtime.spawn(async move {
        async_run.run(move |world| {
            let sender = world.resource(audio_sender());
            sender.send(AudioMessage::StopById(uid)).unwrap();
        });
    });
    Ok(())
}
