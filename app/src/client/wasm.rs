use ambient_audio::Source;
use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_wasm::shared::{get_module_name, MessageType};
use ambient_world_audio::{audio_sender, AudioMessage};
use flume::{Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;

pub fn systems() -> SystemGroup {
    ambient_wasm::client::systems()
}

pub fn initialize(world: &mut World) -> anyhow::Result<()> {
    let messenger = Arc::new(|world: &World, id: EntityId, type_: MessageType, message: &str| {
        let name = get_module_name(world, id);
        let (prefix, level) = match type_ {
            MessageType::Info => ("info", log::Level::Info),
            MessageType::Warn => ("warn", log::Level::Warn),
            MessageType::Error => ("error", log::Level::Error),
            MessageType::Stdout => ("stdout", log::Level::Info),
            MessageType::Stderr => ("stderr", log::Level::Info),
        };

        log::log!(level, "[{name}] {prefix}: {}", message.strip_suffix('\n').unwrap_or(message));
    });

    let (tx, rx): (Sender<AudioMessage>, Receiver<AudioMessage>) = flume::unbounded();

    std::thread::spawn(move || {
        let stream = ambient_audio::AudioStream::new().unwrap();
        let mut sound_info = std::collections::HashMap::new();
        while let Ok(message) = rx.recv() {
            match message {
                AudioMessage::Track(t, looping, amp, url, uid) => {
                    let gain = Arc::new(Mutex::new(amp.clamp(0.0, 1.0)));
                    let gain_clone = gain.clone();

                    let sound = match looping {
                        true => stream.mixer().play(t.decode().repeat().gain(gain_clone)),
                        false => stream.mixer().play(t.decode().gain(gain_clone)),
                    };
                    sound.wait();
                    sound_info.insert(uid, (url, looping, gain, sound.id));
                }
                AudioMessage::UpdateVolume(target_url, amp) => {
                    for (_, (url, _, gain, _)) in sound_info.iter_mut() {
                        if url == &target_url {
                            let mut gain_locked = gain.lock();
                            *gain_locked = amp.clamp(0.0, 1.0);
                        }
                    }
                    // log::info!("Updated volume for all sounds with url {} to {}", target_url, amp);
                }
                AudioMessage::Stop(target_url) => {
                    let mut keys_to_remove: Vec<u32> = Vec::new();

                    for (key, value) in sound_info.iter() {
                        let (url, _, _, _) = value;
                        if url == &target_url {
                            keys_to_remove.push(*key);
                        }
                    }

                    for key in keys_to_remove {
                        let value = sound_info.remove(&key);
                        if let Some((_, _, _, sound_id)) = value {
                            stream.mixer().stop(sound_id);
                        }
                    }
                    log::info!("Stopped all sounds with url {}", target_url);
                }
                AudioMessage::StopById(uid) => {
                    let (_, _, _, id) = match sound_info.remove(&uid) {
                        Some(id) => id,
                        None => {
                            log::error!("No sound with id {}", uid);
                            continue;
                        }
                    };
                    log::info!("Stopped sound with id {}", uid);
                    stream.mixer().stop(id);
                }
            }
        }
    });
    world.add_resource(audio_sender(), Arc::new(tx));

    ambient_wasm::client::initialize(world, messenger)?;

    Ok(())
}
