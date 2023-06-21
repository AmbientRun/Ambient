use crate::shared::{conversion::FromBindgen, wit};
use ambient_audio::Source;
use ambient_audio::{Attenuation, AudioEmitter, AudioFromUrl, AudioListener};
use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    runtime,
    transform::{rotation, translation},
};
use ambient_ecs::{query, World};
use ambient_std::{asset_cache::AsyncAssetKeyExt, asset_url::AbsAssetUrl};
use ambient_world_audio::{audio_emitter, audio_listener, hrtf_lib};
use ambient_world_audio::{audio_sender, AudioMessage};
use anyhow::Context;
use glam::{Mat4, Vec3};
use itertools::Itertools;
use parking_lot::Mutex;
use std::str::FromStr;
use std::sync::Arc;

pub(crate) fn set_listener(world: &mut World, entity: wit::types::EntityId) -> anyhow::Result<()> {
    let pos = world.get(entity.from_bindgen(), translation())?;
    let rotation = world.get(entity.from_bindgen(), rotation())?;
    world.add_component(
        entity.from_bindgen(),
        audio_listener(),
        Arc::new(Mutex::new(AudioListener::new(
            Mat4::from_rotation_translation(rotation, pos),
            Vec3::X * 0.3,
        ))),
    )?;
    Ok(())
}

pub(crate) fn set_emitter(world: &mut World, entity: wit::types::EntityId) -> anyhow::Result<()> {
    let pos = world.get(entity.from_bindgen(), translation())?;
    let emitter = Arc::new(Mutex::new(AudioEmitter {
        amplitude: 5.0,
        attenuation: Attenuation::InversePoly {
            quad: 0.1,
            lin: 0.0,
            constant: 1.0,
        },
        pos,
    }));
    world.add_component(entity.from_bindgen(), audio_emitter(), emitter)?;

    Ok(())
}

pub(crate) fn play_sound_on_entity(
    world: &World,
    sound: String,
    emitter: wit::types::EntityId,
) -> anyhow::Result<()> {
    let assets = world.resource(asset_cache()).clone();
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    let url = AbsAssetUrl::from_str(&sound)?.to_download_url(&assets)?;
    runtime.spawn(async move {
        let track = AudioFromUrl { url: url.clone() }.get(&assets).await;
        async_run.run(move |world| {
            let hrtf_lib = world.resource(hrtf_lib());
            let emitter = world
                .get_ref(emitter.from_bindgen(), audio_emitter())
                .context("No audio emitter on entity")
                .unwrap();
            let (_, listener) = query(audio_listener())
                .iter(world, None)
                .exactly_one()
                .map_err(|v| {
                    anyhow::anyhow!(
                        "Incorrect number of listeners in world. Additional: {:?}",
                        v.count()
                    )
                })
                .unwrap();
            match track {
                Ok(track) => {
                    let sender = world.resource(audio_sender());
                    let source =
                        track
                            .decode()
                            .spatial(hrtf_lib, listener.clone(), emitter.clone());
                    sender.send(AudioMessage::Spatial(source)).unwrap();
                }
                Err(e) => log::error!("{e:?}"),
            };
        });
    });
    Ok(())
}
