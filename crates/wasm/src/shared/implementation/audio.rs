use ambient_std::asset_url::{AbsAssetUrl}; // AssetUrl
use ambient_std::asset_cache::{AsyncAssetKeyExt};
use ambient_audio::{AudioFromUrl}; //  track::TrackDecodeStream, Source
use ambient_ecs::{World};
use ambient_core::{asset_cache};
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
    let track = audio_url.peek(&assets);

    match track {
        Some(track) => {
            let sender = world.resource(audio_sender());
            sender.lock().send(AudioMessage::Track(track.unwrap(), looping, amp)).unwrap();
        },
        None => println!("Track not found or not loaded yet")
    };
    Ok(())
}