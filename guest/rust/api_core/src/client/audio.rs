use crate::{
    core::{
        app::components::name,
        audio::components::*,
        ecs::components::{children, parent},
    },
    prelude::{Entity, EntityId, World},
};

/// stop the audio on the given entity
pub fn stop(world: &mut World, entity: EntityId) {
    if world.exists(entity) {
        world.add_component(entity, stop_now(), ());
    } else {
        eprintln!("Tried to stop audio on non-existent entity {}", entity);
    }
}

/// play spatial audio
#[derive(Debug, Clone)]
pub struct SpatialAudioPlayer {
    /// the entity that represents the spatial audio player
    pub player: EntityId,
}
impl SpatialAudioPlayer {
    pub fn new(world: &mut World) -> Self {
        let player = Entity::new()
            .with(is_spatial_audio_player(), ())
            .with(name(), "Spatial audio player".to_string())
            .spawn(world);
        Self { player }
    }

    pub fn set_listener(&self, world: &mut World, listener: EntityId) {
        world.add_component(self.player, spatial_audio_listener(), listener);
    }

    pub fn set_emitter(&self, world: &mut World, emitter: EntityId) {
        world.add_component(self.player, spatial_audio_emitter(), emitter);
    }

    pub fn set_amplitude(&self, world: &mut World, amp: f32) {
        world.add_component(self.player, amplitude(), amp);
    }

    pub fn set_looping(&self, world: &mut World, val: bool) {
        world.add_component(self.player, looping(), val);
    }

    pub fn play_sound_on_entity(&self, world: &mut World, url: String, emitter: EntityId) {
        world.add_component(self.player, spatial_audio_emitter(), emitter);
        world.add_component(self.player, audio_url(), url);
        world.add_component(self.player, play_now(), ());
    }
}

/// Play the audio file at the given URL.
#[derive(Debug, Clone)]
pub struct AudioPlayer {
    /// The entity that represents the audio player
    pub entity: EntityId,
}
impl AudioPlayer {
    /// Create new audio player from URL
    pub fn new(world: &mut World) -> Self {
        let player = Entity::new()
            .with(is_audio_player(), ())
            .with(name(), "Audio player".to_string())
            .with(children(), vec![])
            .spawn(world);
        Self { entity: player }
    }
    /// Set the sound looping or not
    pub fn set_looping(&self, world: &mut World, val: bool) {
        world.add_component(self.entity, looping(), val);
    }

    /// Add a simple onepole lowpass filter to the sound with one param: roll off frequency
    pub fn add_one_pole_lpf(&self, world: &mut World, rolloff_freq: f32) {
        world.add_component(self.entity, onepole_lpf(), rolloff_freq);
    }

    /// Set the amp/volume of the sound 0.0 is 0%, 1.0 is 100%
    pub fn set_amplitude(&self, world: &mut World, amp: f32) {
        world.add_component(self.entity, amplitude(), amp);
    }
    /// Set the panning of the sound -1.0 is 100% left, 1.0 is 100% right.
    pub fn set_panning(&self, world: &mut World, pan: f32) {
        world.add_component(self.entity, panning(), pan);
    }
    /// Play the sound, this will generate a new entity that represents the playing sound.
    pub fn play(&self, world: &mut World, url: String) -> EntityId {
        world.add_component(self.entity, audio_url(), url);
        world.add_component(self.entity, play_now(), ());
        let id = Entity::new()
            .with(playing_sound(), ())
            .with(name(), "Playing sound".to_string())
            .with(parent(), self.entity)
            .spawn(world);
        world.mutate_component(self.entity, children(), |val| {
            val.push(id);
        });
        id
    }
}
