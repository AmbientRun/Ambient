use ambient_audio::Source;
use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_wasm::shared::{get_module_name, MessageType};
use ambient_world_audio::{audio_sender, AudioFx::*, AudioMessage, SoundInfo};
use flume::{Receiver, Sender};
// use parking_lot::Mutex;
use std::sync::Arc;

pub fn systems() -> SystemGroup {
    ambient_wasm::client::systems()
}

pub fn initialize(world: &mut World) -> anyhow::Result<()> {
    let messenger = Arc::new(
        |world: &World, id: EntityId, type_: MessageType, message: &str| {
            let name = get_module_name(world, id);
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
                    let sound = stream.mixer().play(source);
                    sound.wait();
                }
                AudioMessage::Track {
                    track,
                    url,
                    fx,
                    uid,
                } => {
                    let mut t: Box<dyn Source> = Box::new(track.decode());
                    for effect in &fx {
                        match effect {
                            Panning(pan) => {
                                t = t.pan(*pan);
                            }
                            // Looping => {
                            //     t = t.repeat();
                            // }
                            Amplitude(amp) => {
                                t = t.gain(*amp);
                            }
                            _ => {}
                        }
                    }
                    let sound = stream.mixer().play(t);
                    sound.wait();
                    let sound_info = SoundInfo {
                        url,
                        fx,
                        id: sound.id,
                    };
                    sound_info_lib.insert(uid, sound_info);
                }
                // AudioMessage::UpdateVolume(target_url, amp) => {
                //     for (_, info) in sound_info_lib
                //         .iter_mut()
                //         .filter(|(_, info)| info.url == target_url)
                //     {
                //         *info.volume.lock() = amp;
                //     }
                //     // log::info!("Updated volume for all sounds with url {} to {}", target_url, amp);
                // }
                AudioMessage::Stop(target_url) => {
                    let mut keys_to_remove: Vec<u32> = Vec::new();

                    for (key, info) in sound_info_lib.iter() {
                        if info.url == target_url {
                            keys_to_remove.push(*key);
                        }
                    }

                    for key in keys_to_remove {
                        let info = sound_info_lib.remove(&key);
                        if let Some(info) = info {
                            stream.mixer().stop(info.id);
                        }
                    }
                    // log::info!("Stopped all sounds with url {}", target_url);
                }
                AudioMessage::StopById(uid) => {
                    let id = match sound_info_lib.remove(&uid) {
                        Some(info) => info.id,
                        None => {
                            log::error!("No sound with id {}", uid);
                            continue;
                        }
                    };
                    // log::info!("Stopped sound with id {}", uid);
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
