use std::{io::Cursor, sync::Arc};

use crate::{audio_emitter, audio_listener, hrtf_lib};
use ambient_audio::{hrtf::HrtfLib, AudioFromUrl, Source};
use ambient_audio::{Attenuation, AudioEmitter, AudioListener};
use ambient_core::transform::local_to_world;
use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    runtime,
    transform::{rotation, translation},
};
use ambient_ecs::{generated::components::core::audio::*, query, SystemGroup, World};
use ambient_std::{asset_cache::AsyncAssetKeyExt, asset_url::AbsAssetUrl};
use glam::{vec4, Mat4};
use std::str::FromStr;

pub fn audio_systems() -> SystemGroup {
    SystemGroup::new(
        "audio",
        vec![
            query((audio_player(), trigger_at_this_frame())).to_system(|q, world, qs, _| {
                for (audio_entity, (_, should_play)) in q.collect_cloned(world, qs) {
                    if should_play {
                        let amp = world.get(audio_entity, amplitude()).unwrap_or(1.0);
                        let pan = world.get(audio_entity, panning()).unwrap_or(0.0);
                        let looping = world.get(audio_entity, looping()).unwrap_or(false);
                        world
                            .set(audio_entity, trigger_at_this_frame(), false)
                            .unwrap();
                        let assets = world.resource(asset_cache()).clone();
                        let runtime = world.resource(runtime()).clone();
                        let async_run = world.resource(async_run()).clone();
                        let url = world.get_ref(audio_entity, audio_url()).unwrap();
                        let url = AbsAssetUrl::from_str(url)
                            .unwrap()
                            .to_download_url(&assets)
                            .unwrap();
                        // std::thread::spawn(move || {
                        runtime.spawn(async move {
                            let track = AudioFromUrl { url: url.clone() }.get(&assets).await;
                            async_run.run(move |world| {
                                // log::info!("______playing sound");
                                let sender = world.resource(crate::audio_sender());
                                sender
                                    .send(crate::AudioMessage::Track {
                                        track: track.unwrap(),
                                        url,
                                        fx: if looping {
                                            vec![
                                                crate::AudioFx::Amplitude(amp),
                                                crate::AudioFx::Panning(pan),
                                                crate::AudioFx::Looping,
                                            ]
                                        } else {
                                            vec![
                                                crate::AudioFx::Amplitude(amp),
                                                crate::AudioFx::Panning(pan),
                                            ]
                                        },
                                        uid: 0, //TODO: uid
                                    })
                                    .unwrap();
                            });
                        });
                        // });
                    }
                }
            }),
        ],
    )
}

/// Initializes the HRTF sphere and adds the appropriate resources
///
/// TODO: customizer IR sphere selection
pub fn setup_audio(world: &mut World) -> anyhow::Result<()> {
    let hrtf = Arc::new(HrtfLib::load(Cursor::new(include_bytes!(
        "../IRC_1002_C.bin"
    )))?);
    world.add_resource(hrtf_lib(), hrtf);
    Ok(())
}

/// This translates elements RHS Z-up coordinate system to the HRIR sphere LHS Y-up
/// https://github.com/mrDIMAS/hrir_sphere_builder/blob/e52a10ece678a2b80a0978f7cf23f3ad9cce41c3/src/hrtf_builder.cpp#L155-L162
pub const Y_UP_LHS: Mat4 = Mat4::from_cols(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 0.0, 1.0),
);

pub fn spatial_audio_systems() -> SystemGroup {
    SystemGroup::new(
        "spatial_audio",
        vec![
            query((spatial_audio_player(), trigger_at_this_frame())).to_system(
                |q, world, qs, _| {
                    for (audio_entity, (_, should_play)) in q.collect_cloned(world, qs) {
                        if should_play {
                            let amp = world.get(audio_entity, amplitude()).unwrap_or(1.0);
                            // TODO: find a way to get looping to work
                            // let looping = world.get(audio_entity, looping()).unwrap_or(false);
                            world
                                .set(audio_entity, trigger_at_this_frame(), false)
                                .unwrap();
                            let assets = world.resource(asset_cache()).clone();
                            let runtime = world.resource(runtime()).clone();
                            let async_run = world.resource(async_run()).clone();
                            let url = world.get_ref(audio_entity, audio_url()).unwrap();
                            let url = AbsAssetUrl::from_str(url)
                                .unwrap()
                                .to_download_url(&assets)
                                .unwrap();

                            runtime.spawn(async move {
                                let track = AudioFromUrl { url: url.clone() }.get(&assets).await;
                                async_run.run(move |world| {
                                    // log::info!("______playing sound");
                                    let listener_id =
                                        world.get(audio_entity, spatial_audio_listener()).unwrap();
                                    let emitter_id =
                                        world.get(audio_entity, spatial_audio_emitter()).unwrap();
                                    let pos_listener =
                                        world.get(listener_id, translation()).unwrap();
                                    let rot = world.get(listener_id, rotation()).unwrap();
                                    let pos_emitter = world.get(emitter_id, translation()).unwrap();

                                    let listener =
                                        Arc::new(parking_lot::Mutex::new(AudioListener::new(
                                            Mat4::from_rotation_translation(rot, pos_listener),
                                            glam::Vec3::X * 0.3,
                                        )));
                                    let emitter = Arc::new(parking_lot::Mutex::new(AudioEmitter {
                                        amplitude: amp,
                                        attenuation: Attenuation::InversePoly {
                                            quad: 0.1,
                                            lin: 0.0,
                                            constant: 1.0,
                                        },
                                        pos: pos_emitter,
                                    }));
                                    world
                                        .add_component(emitter_id, audio_emitter(), emitter.clone())
                                        .unwrap();
                                    world
                                        .add_component(
                                            listener_id,
                                            audio_listener(),
                                            listener.clone(),
                                        )
                                        .unwrap();

                                    let sender = world.resource(crate::audio_sender());
                                    let hrtf_lib = world.resource(hrtf_lib());
                                    let source = track
                                        .unwrap()
                                        .decode()
                                        .spatial(hrtf_lib, listener, emitter);
                                    sender.send(crate::AudioMessage::Spatial(source)).unwrap();
                                });
                            });
                        }
                    }
                },
            ),
            // Updates the volume of audio emitters in the world
            query((audio_emitter(), local_to_world())).to_system(|q, world, qs, _| {
                for (_, (emitter, ltw)) in q.iter(world, qs) {
                    let (_, _, pos) = ltw.to_scale_rotation_translation();
                    let mut emitter = emitter.lock();
                    emitter.pos = pos;
                }
            }),
            query((audio_listener(), local_to_world())).to_system_with_name(
                "update_audio_listener",
                |q, world, qs, _| {
                    for (_, (listener, &ltw)) in q.iter(world, qs) {
                        let mut listener = listener.lock();
                        listener.transform = Y_UP_LHS * ltw;
                    }
                },
            ),
        ],
    )
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new("Spatial audio", vec![Box::new(spatial_audio_systems())])
}
