use ambient_audio::Source;
use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_wasm::shared::{module_name, MessageType};
use ambient_world_audio::{audio_sender, AudioControl, AudioFx, AudioMessage, SoundInfo};
use flume::{Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;

pub fn systems() -> SystemGroup {
    ambient_wasm::client::systems()
}

pub fn initialize(world: &mut World, mute_audio: bool) -> anyhow::Result<()> {
    let messenger = Arc::new(
        |world: &World, id: EntityId, type_: MessageType, message: &str| {
            let name = world.get_cloned(id, module_name()).unwrap();
            let (prefix, level) = match type_ {
                MessageType::Info => ("info", log::Level::Info),
                MessageType::Warn => ("warn", log::Level::Warn),
                MessageType::Error => ("error", log::Level::Error),
                MessageType::Stdout => ("stdout", log::Level::Info),
                MessageType::Stderr => ("stderr", log::Level::Info),
            };

            log::log!(
                level,
                "[{name}] {prefix}: {}",
                message.strip_suffix('\n').unwrap_or(message)
            );
        },
    );

    let (tx, rx): (Sender<AudioMessage>, Receiver<AudioMessage>) = flume::unbounded();
    world.add_resource(audio_sender(), Arc::new(tx));

    std::thread::spawn(move || {
        let stream = ambient_audio::AudioStream::new().unwrap();
        let mut sound_info_lib = std::collections::HashMap::new();
        while let Ok(message) = rx.recv() {
            match message {
                AudioMessage::Spatial(source) => {
                    if mute_audio {
                        log::info!("Debug: get a spatial audio message.");
                        continue;
                    }
                    let sound = stream.mixer().play(source);
                    sound.wait();
                }
                AudioMessage::Track {
                    track,
                    url,
                    fx,
                    uid,
                } => {
                    if mute_audio {
                        log::info!("Playing track {}", url);
                        log::info!("Effects: {:?}", fx);
                        continue;
                    }
                    let mut t: Box<dyn Source> = Box::new(track.decode());
                    let mut ctrl = vec![];
                    for effect in &fx {
                        match effect {
                            AudioFx::Panning(pan) => {
                                let p = Arc::new(Mutex::new(*pan));
                                t = t.pan(p.clone());
                                ctrl.push(AudioControl::Panning(p));
                            }
                            AudioFx::Amplitude(amp) => {
                                let a = Arc::new(Mutex::new(*amp));
                                t = t.gain(a.clone());
                                ctrl.push(AudioControl::Amplitude(a));
                            }
                            // Looping => {
                            //     t = t.repeat();
                            // }
                            _ => {}
                        }
                    }
                    let sound = stream.mixer().play(t);
                    sound.wait();
                    let sound_info = SoundInfo {
                        url,
                        control_info: ctrl,
                        id: sound.id,
                    };
                    sound_info_lib.insert(uid, sound_info);
                }
                AudioMessage::UpdateVolume(uid, amp) => {
                    if mute_audio {
                        log::info!("Updating amp for sound with id {} to {}", uid, amp);
                        continue;
                    }
                    let sound = sound_info_lib.get(&uid);
                    if let Some(sound) = sound {
                        for info in &sound.control_info {
                            if let AudioControl::Amplitude(a) = info {
                                *a.lock() = amp;
                            }
                        }
                    }
                }

                AudioMessage::UpdatePanning(uid, pan) => {
                    if mute_audio {
                        log::info!("Updating panning for sound with id {} to {}", uid, pan);
                        continue;
                    }
                    let sound = sound_info_lib.get(&uid);
                    if let Some(sound) = sound {
                        for info in &sound.control_info {
                            if let AudioControl::Panning(p) = info {
                                *p.lock() = pan;
                            }
                        }
                    }
                }
                AudioMessage::StopById(uid) => {
                    if mute_audio {
                        log::info!("Stopped sound with id {}", uid);
                        continue;
                    }
                    let id = match sound_info_lib.remove(&uid) {
                        Some(info) => info.id,
                        None => {
                            log::error!("No sound with id {}", uid);
                            continue;
                        }
                    };
                    stream.mixer().stop(id);
                }
            }
        }
    });

    // TODO: this is not working for some reason
    // let stream = ambient_audio::AudioStream::new().unwrap();
    // world.add_resource(audio_mixer(), stream.mixer().clone());
    ambient_wasm::client::initialize(world, messenger)?;

    Ok(())
}
