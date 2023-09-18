use std::time::Duration;

use crate::{
    core::{
        app::components::name,
        audio::components::*,
        ecs::components::remove_at_game_time,
        hierarchy::components::{children, parent, unmanaged_children},
        transform::components::translation,
    },
    entity,
    prelude::{game_time, Entity, EntityId, Vec3},
};

/// stop the audio on the given entity
pub fn stop(entity: EntityId) {
    if entity::exists(entity) {
        entity::add_component(entity, stop_now(), ());
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

impl Default for SpatialAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialAudioPlayer {
    pub fn new() -> Self {
        let player = Entity::new()
            .with(is_spatial_audio_player(), ())
            .with(name(), "Spatial audio player".to_string())
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

    pub fn play_sound_on_entity(&self, url: impl Into<String>, emitter: EntityId) {
        entity::add_component(self.player, spatial_audio_emitter(), emitter);
        entity::add_component(self.player, audio_url(), url.into());
        entity::add_component(self.player, play_now(), ());
    }
}
impl SpatialAudioPlayer {
    /// Plays a sound at the given position. Note that the returned [`SpatialAudioPlayer`]
    /// will be removed after 60 seconds.
    ///
    /// If no camera is available as a listener, no player will be created.
    // TODO: Should we encourage use of this API? It's temporary, but it's also a lot easier to use.
    pub fn oneshot(position: Vec3, url: impl Into<String>) -> Option<SpatialAudioPlayer> {
        let Some(active_camera) = crate::camera::get_active(None) else {
            return None;
        };

        let player = SpatialAudioPlayer::new();
        entity::add_component(player.player, translation(), position);
        entity::add_component(
            player.player,
            remove_at_game_time(),
            game_time() + Duration::from_secs(60),
        );
        player.set_listener(active_camera);
        player.play_sound_on_entity(url.into(), player.player);

        Some(player)
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
            .with(is_audio_player(), ())
            .with(name(), "Audio player".to_string())
            .with(children(), vec![])
            .with(unmanaged_children(), ())
            .spawn();
        Self { entity: player }
    }
    /// Set the sound looping or not
    pub fn set_looping(&self, val: bool) {
        entity::add_component(self.entity, looping(), val);
    }

    /// Add a simple onepole lowpass filter to the sound with one param: roll off frequency
    pub fn add_one_pole_lpf(&self, rolloff_freq: f32) {
        entity::add_component(self.entity, onepole_lpf(), rolloff_freq);
    }

    /// Set the amp/volume of the sound 0.0 is 0%, 1.0 is 100%
    pub fn set_amplitude(&self, amp: f32) {
        entity::add_component(self.entity, amplitude(), amp);
    }
    /// Set the panning of the sound -1.0 is 100% left, 1.0 is 100% right.
    pub fn set_panning(&self, pan: f32) {
        entity::add_component(self.entity, panning(), pan);
    }
    /// Play the sound, this will generate a new entity that represents the playing sound.
    pub fn play(&self, url: String) -> EntityId {
        entity::add_component(self.entity, audio_url(), url);
        entity::add_component(self.entity, play_now(), ());
        let id = Entity::new()
            .with(playing_sound(), ())
            .with(name(), "Playing sound".to_string())
            .with(parent(), self.entity)
            .spawn();
        entity::mutate_component(self.entity, children(), |val| {
            val.push(id);
        });
        id
    }
}
