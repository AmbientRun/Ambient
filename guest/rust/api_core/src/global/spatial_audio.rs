use crate::{
    global::{EntityId},
    internal::{
        conversion::{IntoBindgen},
        wit,
    },
};

/// Sets the entity that will be used as the listener for positional audio.
pub fn set_listener(entity: EntityId) {
    wit::world_audio::set_listener(entity.into_bindgen());
}

/// Sets the entity that will be used as the emitter for positional audio.
pub fn set_emitter(entity: EntityId) {
    wit::world_audio::set_emitter(entity.into_bindgen());
}

/// Plays a sound on the given entity.
pub fn play_sound_on_entity(url: String, emitter: EntityId) {
    wit::world_audio::play_sound_on_entity(&url, emitter.into_bindgen());
}