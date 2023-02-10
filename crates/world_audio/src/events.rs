use kiwi_core::asset_cache;
use kiwi_ecs::{EntityId, World};

use crate::{play_sound_on_entity, AudioNode, AudioSeed};
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
/// Plays a sound on an entity. Requires that the entity has an AudioEmitterDef on the server.
/// Otherwise, the audio is played on a temporary emitter
pub struct PlayLocalSound {
    pub id: EntityId,
    pub source: AudioNode,
    /// A human readable label describing what this sound is
    pub label: String,
    pub seed: AudioSeed,
}

pub fn play_local_sound(world: &mut World, event: PlayLocalSound) -> anyhow::Result<()> {
    let assets = world.resource(asset_cache());
    let source = match event.source.try_build(assets, event.seed).transpose() {
        Some(source) => source?,
        None => {
            tracing::warn!("Sound {} is not yet loaded", event.label);
            return Ok(());
        }
    };

    play_sound_on_entity(world, event.id, source)?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct StopLocalSound {
    pub id: EntityId,
    pub name: String,
}

pub fn stop_local_sound(_world: &mut World, _event: StopLocalSound) -> anyhow::Result<()> {
    anyhow::bail!("stopping a sound is not supported")
}
