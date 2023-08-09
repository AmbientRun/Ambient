use ambient_audio::AudioMixer;
use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_wasm::shared::{module_name, MessageType};

use std::sync::Arc;

pub fn systems() -> SystemGroup {
    ambient_wasm::client::systems()
}

pub fn initialize(world: &mut World, mixer: Option<AudioMixer>) -> anyhow::Result<()> {
    let messenger = Arc::new(
        |world: &World, id: EntityId, type_: MessageType, message: &str| {
            let name = world.get_cloned(id, module_name()).unwrap_or_default();
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

    if let Some(mixer) = mixer {
        world.add_resource(ambient_world_audio::audio_mixer(), mixer);
    }

    ambient_wasm::client::initialize(world, messenger)?;

    Ok(())
}
