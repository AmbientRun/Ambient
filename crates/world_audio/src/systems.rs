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
use ambient_ecs::{
    children, generated::components::core::audio::*, parent, query, SystemGroup, World,
};
use ambient_native_std::{asset_cache::AsyncAssetKeyExt, asset_url::AbsAssetUrl};
use glam::{vec4, Mat4};
use parking_lot::Mutex;
use std::str::FromStr;

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
/// <https://github.com/mrDIMAS/hrir_sphere_builder/blob/e52a10ece678a2b80a0978f7cf23f3ad9cce41c3/src/hrtf_builder.cpp#L155-L162>
pub const Y_UP_LHS: Mat4 = Mat4::from_cols(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 0.0, 1.0),
);

pub fn audio_systems() -> SystemGroup {
    SystemGroup::new(
        "audio",
        vec![
            query((spatial_audio_player(), play_now())).to_system(|q, world, qs, _| {
                for (audio_player_enitty, _) in q.collect_cloned(world, qs) {
                    let amp = world.get(audio_player_enitty, amplitude()).unwrap_or(1.0);

                    world
                        .remove_component(audio_player_enitty, play_now())
                        .unwrap();

                    let assets = world.resource(asset_cache()).clone();
                    let runtime = world.resource(runtime()).clone();
                    let async_run = world.resource(async_run()).clone();
                    let url = world.get_ref(audio_player_enitty, audio_url()).unwrap();
                    let url = AbsAssetUrl::from_str(url)
                        .unwrap()
                        .to_download_url(&assets)
                        .unwrap();

                    runtime.spawn(async move {
                        let track = AudioFromUrl { url: url.clone() }.get(&assets).await;
                        async_run.run(move |world| {
                            let listener_id = world
                                .get(audio_player_enitty, spatial_audio_listener())
                                .unwrap();
                            let emitter_id = world
                                .get(audio_player_enitty, spatial_audio_emitter())
                                .unwrap();
                            let pos_listener = world.get(listener_id, translation()).unwrap();
                            let rot = world.get(listener_id, rotation()).unwrap();
                            let pos_emitter = world.get(emitter_id, translation()).unwrap();

                            let listener = Arc::new(parking_lot::Mutex::new(AudioListener::new(
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
                                .add_component(listener_id, audio_listener(), listener.clone())
                                .unwrap();

                            let hrtf_lib = world.resource(hrtf_lib());
                            let _source =
                                track.unwrap().decode().spatial(hrtf_lib, listener, emitter);
                            // TODO: find a way to get looping to work

                        });
                    });
                }
            }),
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
            query((playing_sound(), stop_now())).to_system(|q, world, qs, _| {
                for (playing_entity, _) in q.collect_cloned(world, qs) {
                    let mixer = world.resource(crate::audio_mixer());
                    let id = world.get(playing_entity, crate::sound_id());
                    if id.is_err() {
                        log::error!("No sound id component on playing entity; cannot stop audio.");
                        continue;
                    }
                    mixer.stop(id.unwrap());
                    let p = world.get(playing_entity, parent());
                    if p.is_err() {
                        log::error!("No parent component on playing entity; cannot stop audio.");
                        continue;
                    }
                    let parent_entity = p.unwrap();

                    let c = world.get_ref(parent_entity, children());
                    if c.is_err() {
                        log::error!("No children component on parent entity; cannot stop audio.");
                        continue;
                    }
                    let new_children = c
                        .unwrap()
                        .iter()
                        .filter(|&&e| e != playing_entity)
                        .cloned()
                        .collect::<Vec<_>>();
                    world.set(parent_entity, children(), new_children).unwrap();
                    world.despawn(playing_entity);
                }
            }),
            query((playing_sound(), amplitude())).to_system(|q, world, qs, _| {
                for (playing_entity, (_, amp)) in q.collect_cloned(world, qs) {
                    let amp_arc = world
                        .get_mut(playing_entity, crate::amplitude_arc())
                        .unwrap();
                    *amp_arc.lock() = amp;
                }
            }),
            query((playing_sound(), panning())).to_system(|q, world, qs, _| {
                for (playing_entity, (_, pan)) in q.collect_cloned(world, qs) {
                    let pan_arc = world
                        .get_mut(playing_entity, crate::panning_arc())
                        .unwrap();
                    *pan_arc.lock() = pan;
                }
            }),
            query((playing_sound(), onepole_lpf())).to_system(|q, world, qs, _| {
                for (playing_entity, (_, freq)) in q.collect_cloned(world, qs) {
                    let freq_arc = world
                        .get_mut(playing_entity, crate::onepole_arc())
                        .unwrap();
                    *freq_arc.lock() = freq;
                }
            }),
            query((audio_player(), play_now(), audio_url())).to_system(|q, world, qs, _| {
                for (audio_player_enitty, (_, _, url)) in q.collect_cloned(world, qs) {
                    let amp = world.get(audio_player_enitty, amplitude()).unwrap_or(1.0);
                    let pan = world.get(audio_player_enitty, panning()).unwrap_or(0.0);
                    let freq = world
                        .get(audio_player_enitty, onepole_lpf())
                        .unwrap_or(20000.0);
                    let looping = world.get(audio_player_enitty, looping()).unwrap_or(false);

                    world
                        .remove_component(audio_player_enitty, play_now())
                        .unwrap();

                    let assets = world.resource(asset_cache()).clone();
                    let runtime = world.resource(runtime()).clone();
                    let async_run = world.resource(async_run()).clone();
                    let url = AbsAssetUrl::from_str(&url)
                        .unwrap()
                        .to_download_url(&assets)
                        .unwrap();

                    runtime.spawn(async move {
                        let track = AudioFromUrl { url: url.clone() }.get(&assets).await;
                        let track = track.unwrap();
                        let id_arc = Arc::new(Mutex::new(None));
                        let id_arc_clone = id_arc.clone();
                        let count_arc = Arc::new(Mutex::new(None));
                        let count_arc_clone = count_arc.clone();
                        let sr_arc = Arc::new(Mutex::new(None));
                        let sr_arc_clone = sr_arc.clone();
                        async_run.run(move |world| {
                            let id_vec = world.get_ref(audio_player_enitty, children());
                            if id_vec.is_err() {
                                log::error!(
                                    "No children component on parent entity; cannot play audio."
                                );
                                return;
                            }
                            let id_vec = id_vec.unwrap();
                            if id_vec.is_empty() {
                                log::error!(
                                    "No children component on parent entity; cannot play audio."
                                );
                                return;
                            }
                            let id = id_vec.last().unwrap();
                            id_arc.lock().replace(*id);

                            let mut t: Box<dyn Source> = if looping {
                                Box::new(track.decode().repeat())
                            } else {
                                let decoded = track.decode();
                                let count = decoded.sample_count().unwrap();
                                let sr = decoded.sample_rate();
                                *count_arc.lock() = Some(count);
                                *sr_arc.lock() = Some(sr);
                                Box::new(decoded)
                            };
                            let a = Arc::new(Mutex::new(amp));
                            t = t.gain(a.clone());
                            let p = Arc::new(Mutex::new(pan));
                            t = t.pan(p.clone());
                            let f = Arc::new(Mutex::new(freq));
                            t = t.onepole(f.clone());

                            let id = id_arc.lock().unwrap();
                            world.add_component(id, crate::amplitude_arc(), a).unwrap();
                            world.add_component(id, crate::panning_arc(), p).unwrap();
                            world.add_component(id, crate::onepole_arc(), f).unwrap();

                            let mixer = world.resource(crate::audio_mixer());
                            let sound = mixer.play(t);

                            world.add_component(id, crate::sound_id(), sound.id).unwrap();
                        });

                        let count = *count_arc_clone.lock();
                        let sr = *sr_arc_clone.lock();
                        if count.is_some() && sr.is_some() {
                            let dur = count.unwrap() as f32 / sr.unwrap() as f32 * 1.001;
                            ambient_sys::time::sleep(std::time::Duration::from_secs_f32(dur)).await;
                            async_run.run(move |world| {
                                world.despawn(id_arc_clone.lock().unwrap());
                                if !world.exists(audio_player_enitty) {
                                    return;
                                }
                                let child = world.get_ref(audio_player_enitty, children());
                                if child.is_err() {
                                    log::error!("No children component on parent entity; cannot auto stop audio.");
                                    return;
                                }
                                let new_child = child
                                    .unwrap()
                                    .iter()
                                    .filter(|c| *c != &id_arc_clone.lock().unwrap())
                                    .cloned()
                                    .collect();
                                world.set(audio_player_enitty, children(), new_child).unwrap();
                            });
                        };
                    });
                }
            }),
        ],
    )
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new("audio", vec![Box::new(audio_systems())])
}
