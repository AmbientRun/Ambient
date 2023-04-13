use std::sync::Arc;

use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_wasm::shared::{get_module_name, MessageType};
use std::sync::mpsc::{self, Sender, Receiver};
use ambient_world_audio::{AudioMessage, audio_sender};
use parking_lot::Mutex;
use ambient_audio::Source;

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

    let (tx, rx): (Sender<AudioMessage>, Receiver<AudioMessage>) = mpsc::channel();

    std::thread::spawn(move || {
        let stream = ambient_audio::AudioStream::new().unwrap();
        while let Ok(message) = rx.recv() {
            match message {
                AudioMessage::Track(t, looping, amp) => {
                    // println!("got message track {:?}", t.decode().duration());
                    match looping {
                        true => {
                            let sound = stream.mixer().play(t.decode().repeat().gain(amp.clamp(0.0, 1.0)));
                            sound.wait();
                        }
                        false => {
                            let sound = stream.mixer().play(t.decode().gain(amp.clamp(0.0, 1.0)));
                            sound.wait();
                        }
                    }
                }
            }
        }
    });
    world.add_resource(audio_sender(), Arc::new(Mutex::new(tx)));

    ambient_wasm::client::initialize(world, messenger)?;

    Ok(())
}
