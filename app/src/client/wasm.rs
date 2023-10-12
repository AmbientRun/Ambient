use ambient_audio::AudioMixer;
use ambient_ecs::{EntityId, SystemGroup, World};
use ambient_native_std::asset_cache::AssetCache;
use ambient_wasm::shared::{module_name, MessageType};

use std::sync::Arc;

pub fn systems() -> SystemGroup {
    ambient_wasm::client::systems()
}

pub fn initialize(
    world: &mut World,
    assets: &AssetCache,
    mixer: Option<AudioMixer>,
) -> anyhow::Result<()> {
    let messenger = Arc::new(
        |world: &World, id: EntityId, ty: MessageType, message: &str| {
            let module_name = world.get_cloned(id, module_name()).unwrap_or_default();

            match ty {
                MessageType::Info => tracing::info!(%module_name, "{}", message),
                MessageType::Warn => tracing::warn!(%module_name, "{}", message),
                MessageType::Error => tracing::error!(%module_name, "{}", message),
                MessageType::Stdout => tracing::info!(%module_name, "stdout: {}", message),
                MessageType::Stderr => tracing::info!(%module_name, "stderr: {}", message),
            };
        },
    );

    if let Some(mixer) = mixer {
        world.add_resource(ambient_world_audio::audio_mixer(), mixer);
    }

    ambient_wasm::client::initialize(world, assets, messenger)?;

    Ok(())
}
