use std::{io::Cursor, sync::Arc};

use crate::{audio_emitter, audio_listener, hrtf_lib};
use ambient_audio::{hrtf::HrtfLib, AudioFromUrl, Source};
use ambient_audio::{Attenuation, AudioEmitter, AudioListener};
use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    runtime,
    transform::{local_to_world, translation},
};
use ambient_ecs::EntityId;
use ambient_ecs::{
    generated::audio::components::*, generated::hierarchy::components::children, query,
    SystemGroup, World,
};
use ambient_native_std::{asset_cache::AsyncAssetKeyExt, asset_url::AbsAssetUrl, unwrap_log_warn};
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
            query(audio_url())
                .incl(is_spatial_audio_player())
                .incl(play_now())
                .to_system(|q, world, qs, _| {
                    for (player, url) in q.collect_cloned(world, qs) {
                        process_player(world, player, &url);
                    }
                }),
            // Updates the volume of audio emitters in the world
            query((audio_emitter(), local_to_world())).to_system(|q, world, qs, _| {
                for (_, (emitter, ltw)) in q.iter(world, qs) {
                    // check if mute_audio is set
                    let r = world.resource_entity();
                    if !world.has_component(r, crate::audio_mixer()) {
                        continue;
                    }
                    let (_, _, pos) = ltw.to_scale_rotation_translation();
                    let mut emitter = emitter.lock();
                    emitter.pos = pos;
                }
            }),
            query((audio_listener(), local_to_world())).to_system_with_name(
                "update_audio_listener",
                |q, world, qs, _| {
                    for (_, (listener, &ltw)) in q.iter(world, qs) {
                        // check if mute_audio is set
                        let r = world.resource_entity();
                        if !world.has_component(r, crate::audio_mixer()) {
                            continue;
                        }
                        let mut listener = listener.lock();
                        listener.transform = Y_UP_LHS * ltw;
                    }
                },
            ),
            query(stop_now()).to_system(|q, world, qs, _| {
                for (playing_entity, _) in q.collect_cloned(world, qs) {
                    // check if mute_audio is set
                    let r = world.resource_entity();
                    if !world.has_component(r, crate::audio_mixer()) {
                        continue;
                    }

                    let mixer = world.resource(crate::audio_mixer());
                    let Ok(id) = world.get(playing_entity, crate::sound_id()) else {
                        log::error!("No sound id component on playing entity; cannot stop audio.");
                        continue;
                    };
                    mixer.stop(id);
                    // stopping an emitter is different
                    if world.has_component(playing_entity, audio_emitter()) {
                        let _ = world.remove_component(playing_entity, audio_emitter());
                        continue;
                    }
                    if world.has_component(playing_entity, audio_listener()) {
                        let _ = world.remove_component(playing_entity, audio_listener());
                        continue;
                    }
                    let _ = world.remove_component(playing_entity, stop_now());

                    world.despawn(playing_entity);
                }
            }),
            query((playing_sound(), amplitude())).to_system(|q, world, qs, _| {
                for (playing_entity, (_, amp)) in q.collect_cloned(world, qs) {
                    // check if mute_audio is set
                    let r = world.resource_entity();
                    if !world.has_component(r, crate::audio_mixer()) {
                        continue;
                    }
                    if let Ok(amp_arc) = world.get_mut(playing_entity, crate::amplitude_arc()) {
                        *amp_arc.lock() = amp;
                    }
                }
            }),
            query((playing_sound(), panning())).to_system(|q, world, qs, _| {
                for (playing_entity, (_, pan)) in q.collect_cloned(world, qs) {
                    // check if mute_audio is set
                    let r = world.resource_entity();
                    if !world.has_component(r, crate::audio_mixer()) {
                        continue;
                    }
                    if let Ok(pan_arc) = world.get_mut(playing_entity, crate::panning_arc()) {
                        *pan_arc.lock() = pan;
                    }
                }
            }),
            query((playing_sound(), onepole_lpf())).to_system(|q, world, qs, _| {
                for (playing_entity, (_, freq)) in q.collect_cloned(world, qs) {
                    // check if mute_audio is set
                    let r = world.resource_entity();
                    if !world.has_component(r, crate::audio_mixer()) {
                        continue;
                    }
                    if let Ok(freq_arc) = world.get_mut(playing_entity, crate::onepole_arc()) {
                        *freq_arc.lock() = freq;
                    }
                }
            }),
            query((is_audio_player(), play_now(), audio_url())).to_system(|q, world, qs, _| {
                for (player, (_, _, url)) in q.collect_cloned(world, qs) {
                    // check if mute_audio is set
                    let r = world.resource_entity();
                    if !world.has_component(r, crate::audio_mixer()) {
                        continue;
                    }

                    let amp = world.get(player, amplitude()).unwrap_or(1.0);
                    let pan = world.get(player, panning()).unwrap_or(0.0);
                    let freq = world.get(player, onepole_lpf()).unwrap_or(20000.0);
                    let looping = world.get(player, looping()).unwrap_or(false);

                    world.remove_component(player, play_now()).unwrap();

                    let assets = world.resource(asset_cache()).clone();
                    let runtime = world.resource(runtime()).clone();
                    let async_run = world.resource(async_run()).clone();
                    let Ok(url) =
                        AbsAssetUrl::from_str(&url).and_then(|u| u.to_download_url(&assets))
                    else {
                        continue;
                    };

                    runtime.spawn(async move {
                        let track =
                            unwrap_log_warn!(AudioFromUrl { url: url.clone() }.get(&assets).await);
                        let id_arc = Arc::new(Mutex::new(None));
                        let id_arc_clone = id_arc.clone();
                        let count_arc = Arc::new(Mutex::new(None));
                        let count_arc_clone = count_arc.clone();
                        let sr_arc = Arc::new(Mutex::new(None));
                        let sr_arc_clone = sr_arc.clone();
                        async_run.run(move |world| {
                            let Some(id) = world
                                .get_ref(player, children())
                                .ok()
                                .and_then(|c| c.last())
                                .copied()
                            else {
                                log::error!(
                                    "No children component on parent entity; cannot play audio."
                                );
                                return;
                            };
                            id_arc.lock().replace(id);

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
                            let _ = world.add_component(id, crate::amplitude_arc(), a);
                            let _ = world.add_component(id, crate::panning_arc(), p);
                            let _ = world.add_component(id, crate::onepole_arc(), f);

                            let mixer = world.resource(crate::audio_mixer());
                            let sound = mixer.play(t);

                            let _ = world.add_component(id, crate::sound_id(), sound.id);
                        });

                        let count = *count_arc_clone.lock();
                        let sr = *sr_arc_clone.lock();
                        if let Some((count, sr)) = count.zip(sr) {
                            let dur = count as f32 / sr as f32 * 1.001;
                            ambient_sys::time::sleep(std::time::Duration::from_secs_f32(dur)).await;
                            async_run.run(move |world| {
                                if let Some(id) = id_arc_clone.lock().take() {
                                    world.despawn(id);
                                }
                            });
                        };
                    });
                }
            }),
        ],
    )
}

fn process_player(world: &mut World, player: EntityId, url: &str) {
    // check if mute_audio is set
    let r = world.resource_entity();
    if !world.has_component(r, crate::audio_mixer()) {
        return;
    }

    let amp = world.get(player, amplitude()).unwrap_or(1.0);
    let looping = world.get(player, looping()).unwrap_or(false);
    world.remove_component(player, play_now()).unwrap();

    let assets = world.resource(asset_cache()).clone();
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    let url = unwrap_log_warn!(AbsAssetUrl::from_str(url).and_then(|u| u.to_download_url(&assets)));

    runtime.spawn(async move {
        let track = unwrap_log_warn!(AudioFromUrl { url: url.clone() }.get(&assets).await);
        async_run.run(move |world| {
            let Ok(listener_id) = world.get(player, spatial_audio_listener()) else {
                return;
            };
            let Ok(emitter_id) = world.get(player, spatial_audio_emitter()) else {
                return;
            };
            let Ok(listener_transform) = world.get(listener_id, local_to_world()) else {
                return;
            };

            let pos_emitter = world.get(emitter_id, translation()).unwrap();

            let listener = Arc::new(Mutex::new(AudioListener::new(
                listener_transform,
                glam::Vec3::X * 0.3,
            )));
            let emitter = Arc::new(Mutex::new(AudioEmitter {
                amplitude: amp,
                attenuation: Attenuation::InversePoly {
                    quad: 0.1,
                    lin: 0.0,
                    constant: 1.0,
                },
                pos: pos_emitter,
            }));
            let _ = world.add_component(emitter_id, audio_emitter(), emitter.clone());
            let _ = world.add_component(listener_id, audio_listener(), listener.clone());

            let hrtf_lib = world.resource(hrtf_lib());

            let mixer = world.resource(crate::audio_mixer());
            let source: Box<dyn Source> = if looping {
                Box::new(track.decode().repeat().spatial(hrtf_lib, listener, emitter))
            } else {
                Box::new(track.decode().spatial(hrtf_lib, listener, emitter))
            };
            let sound = mixer.play(source);
            let _ = world.add_component(emitter_id, crate::sound_id(), sound.id);
        });
    });
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new("audio", vec![Box::new(audio_systems())])
}
