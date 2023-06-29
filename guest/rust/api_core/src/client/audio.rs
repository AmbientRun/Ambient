use crate::{
    components::core::{
        app::name,
        audio::{
            amplitude, audio_player, audio_url, looping, panning, spatial_audio_emitter,
            spatial_audio_listener, spatial_audio_player, trigger_at_this_frame,
        },
    },
    entity,
    prelude::{Entity, EntityId},
};

/// play spatial audio
#[derive(Debug, Clone)]
pub struct SpatialAudioPlayer {
    /// the entity that represents the spatial audio player
    pub player: EntityId,
}

impl Default for SpatialAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialAudioPlayer {
    pub fn new() -> Self {
        let player = Entity::new()
            .with_default(spatial_audio_player())
            .with(name(), "Spatial audio player".to_string())
            .with(trigger_at_this_frame(), false)
            .spawn();
        Self { player }
    }

    pub fn set_listener(&self, listener: EntityId) {
        entity::add_component(self.player, spatial_audio_listener(), listener);
    }

    pub fn set_emitter(&self, emitter: EntityId) {
        entity::add_component(self.player, spatial_audio_emitter(), emitter);
    }

    pub fn set_amplitude(&self, amp: f32) {
        entity::add_component(self.player, amplitude(), amp);
    }

    pub fn set_looping(&self, val: bool) {
        entity::add_component(self.player, looping(), val);
    }

    pub fn play(&self, url: String) {
        entity::add_component(self.player, audio_url(), url);
        entity::set_component(self.player, trigger_at_this_frame(), true);
    }

    pub fn play_sound_on_entity(&self, url: String, emitter: EntityId) {
        entity::add_component(self.player, spatial_audio_emitter(), emitter);
        entity::add_component(self.player, audio_url(), url);
        entity::set_component(self.player, trigger_at_this_frame(), true);
    }
}

/// Play the audio file at the given URL.
#[derive(Debug, Clone)]
pub struct AudioPlayer {
    /// The entity that represents the audio player
    pub entity: EntityId,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPlayer {
    /// Create new audio player from URL
    pub fn new() -> Self {
        let player = Entity::new()
            .with_default(audio_player())
            .with(name(), "Audio player".to_string())
            .with(trigger_at_this_frame(), false)
            .spawn();
        Self { entity: player }
    }
    /// set the looping of the sound
    pub fn set_looping(&self, val: bool) {
        entity::add_component(self.entity, looping(), val);
    }

    /// set the amp/volume of the sound
    pub fn set_amplitude(&self, amp: f32) {
        entity::add_component(self.entity, amplitude(), amp);
    }
    /// set the panning of the sound -1.0 is 100% left, 1.0 is 100% right.
    pub fn set_panning(&self, pan: f32) {
        entity::add_component(self.entity, panning(), pan);
    }
    /// play the sound
    pub fn play(&self, url: String) {
        entity::add_component(self.entity, audio_url(), url);
        entity::set_component(self.entity, trigger_at_this_frame(), true);
    }
}
