use ambient_core::asset_cache;
// use ambient_audio::AudioMixer;
use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_wasm::shared::{module_name, MessageType};
use tracing::Level;

use std::sync::Arc;

pub fn systems() -> SystemGroup {
    ambient_wasm::client::systems()
}

pub fn initialize(world: &mut World) -> anyhow::Result<()> {
    let assets = world.resource(asset_cache()).clone();

    let messenger = Arc::new(
        |world: &World, id: EntityId, type_: MessageType, message: &str| {
            let name = world.get_cloned(id, module_name()).unwrap_or_default();
            let (prefix, level) = match type_ {
                MessageType::Info => ("info", Level::INFO),
                MessageType::Warn => ("warn", Level::WARN),
                MessageType::Error => ("error", Level::ERROR),
                MessageType::Stdout => ("stdout", Level::INFO),
                MessageType::Stderr => ("stderr", Level::INFO),
            };

            tracing::event!(
                Level::INFO,
                "{prefix}: {}",
                message.strip_suffix('\n').unwrap_or(message)
            );
        },
    );

    // TODO: audio
    // if let Some(mixer) = mixer {
    //     world.add_resource(ambient_world_audio::audio_mixer(), mixer);
    // }

    ambient_wasm::client::initialize(world, &assets, messenger)?;

    Ok(())
}
