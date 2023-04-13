use ambient_std::asset_url::{AbsAssetUrl}; // AssetUrl
use ambient_std::asset_cache::{AsyncAssetKeyExt};
use ambient_audio::{AudioFromUrl}; //  track::TrackDecodeStream, Source
use ambient_ecs::{World};
use ambient_core::{asset_cache, async_ecs::async_run, runtime};
use ambient_world_audio::{audio_sender, AudioMessage}; // audio_tracks,

pub(crate) fn add_track(world: &World, url: String) -> anyhow::Result<()> {
    let assets = world.resource(asset_cache()).clone();
    let asset_url = AbsAssetUrl::from_asset_key(url).to_string();
    let audio_url = AudioFromUrl { url: AbsAssetUrl::parse(asset_url).expect("Failed to parse audio url") };
    let _track = audio_url.peek(&assets);
    Ok(())
}

pub(crate) fn play(world: &World, url: String, looping: bool, amp: f32) -> anyhow::Result<()> {
    let assets = world.resource(asset_cache()).clone();
    let asset_url = AbsAssetUrl::from_asset_key(url).to_string();
    let audio_url = AudioFromUrl { url: AbsAssetUrl::parse(asset_url).expect("Failed to parse audio url") };
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    runtime.spawn(async move {
        let track = audio_url.get(&assets).await;
        async_run.run(move |world| {
            match track {
                Ok(track) => {
                    let sender = world.resource(audio_sender());
                    sender.lock().send(AudioMessage::Track(track, looping, amp)).unwrap();
                },
                Err(e) => eprintln!("{e:?}")
            };
        });
    });
    Ok(())
}