use ambient_std::asset_url::{AbsAssetUrl}; // AssetUrl
use ambient_std::asset_cache::{AsyncAssetKeyExt};
use ambient_audio::{AudioFromUrl}; //  track::TrackDecodeStream, Source
use ambient_ecs::{World};
use ambient_core::{asset_cache, async_ecs::async_run, runtime};
use ambient_world_audio::{audio_tracks, audio_sender, AudioMessage}; // audio_tracks,

pub(crate) fn add_track(world: &World, name: String, url: String) -> anyhow::Result<()> {
    let assets = world.resource(asset_cache()).clone();
    let asset_url = AbsAssetUrl::from_asset_key(url).to_string().replace("ambient-assets:/", "");
    println!("Loading sound from: {:?}", asset_url);
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    runtime.spawn(async move {
        let audio_url = AudioFromUrl { url: AbsAssetUrl::parse(asset_url).expect("Failed to parse audio url") };
        let track = audio_url.get(&assets).await.expect("Failed to load audio");
        async_run.run(move |world| {
            match world.resource_mut_opt(audio_tracks()) {
                Some(lib) => {
                    lib.insert(name, track);
                },
                None => {
                    let mut lib = std::collections::HashMap::new();
                    lib.insert(name, track);
                    world.add_resource(audio_tracks(), lib);
                }
            };
        });
    });
    Ok(())
}

pub(crate) fn play(world: &World, name: String, looping: bool, amp: f32) -> anyhow::Result<()> {
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    runtime.spawn(async move {
        async_run.run(move |world| {
            match world.resource_mut_opt(audio_tracks()) {
                Some(lib) => {
                    let track = lib.get(&name).unwrap().clone(); // todo: handle error
                    // println!("duration {:?}", track.decode().duration());
                    let sender = world.resource_mut(audio_sender());
                    sender.lock().send(AudioMessage::Track(track, looping, amp)).unwrap();
                },
                None => {
                    println!("you need to add the sound first");
                }
            };
        });
    });
    Ok(())
}