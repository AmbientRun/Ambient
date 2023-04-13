use ambient_std::asset_url::{AbsAssetUrl}; // AssetUrl
use ambient_std::asset_cache::{AsyncAssetKeyExt};
use ambient_audio::{AudioFromUrl}; //  track::TrackDecodeStream, Source
use ambient_ecs::{World};
use ambient_core::{asset_cache, async_ecs::async_run, runtime};
use ambient_world_audio::{audio_sender, AudioMessage}; // audio_tracks,
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub(crate) fn add_track(world: &World, url: String) -> anyhow::Result<()> {
    let assets = world.resource(asset_cache()).clone();
    let asset_url = AbsAssetUrl::from_asset_key(url).to_string();
    let audio_url = AudioFromUrl { url: AbsAssetUrl::parse(asset_url).expect("Failed to parse audio url") };
    let _track = audio_url.peek(&assets);
    Ok(())
}

pub(crate) fn play(world: &World, url: String, looping: bool, amp: f32) -> anyhow::Result<()> {
    let assets = Arc::new(world.resource(asset_cache()).clone());
    let asset_url = AbsAssetUrl::from_asset_key(url).to_string();
    let audio_url = Arc::new(
        AudioFromUrl { url: AbsAssetUrl::parse(asset_url).expect("Failed to parse audio url") }
    );
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    let break_flag = Arc::new(AtomicBool::new(false));
    runtime.spawn(async move {
        for _ in 0..1000 {
            if break_flag.load(Ordering::SeqCst) {
                break;
            }
            let break_flag_clone = break_flag.clone();
            let audio_url_clone = audio_url.clone();
            let assets_clone = assets.clone();
            async_run.run(move |world| {
                let track = audio_url_clone.peek(&*assets_clone);
                match track {
                    Some(track) => {
                        let sender = world.resource(audio_sender());
                        sender.lock().send(AudioMessage::Track(track.unwrap(), looping, amp)).unwrap();
                        break_flag_clone.store(true, Ordering::SeqCst);
                    },
                    None => {}
                };
            });
            tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.1)).await;
        }
        if !break_flag.load(Ordering::SeqCst) {
            eprintln!("Audio track not found.");
        }
    });
    Ok(())
}