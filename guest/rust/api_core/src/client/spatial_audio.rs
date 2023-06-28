use crate::{
    global::EntityId,
    internal::{conversion::IntoBindgen, wit},
};

/// Sets the entity that will be used as the listener for positional audio.
pub fn set_listener(entity: EntityId) {
    wit::world_audio::set_listener(entity.into_bindgen());
}

/// Sets the entity that will be used as the emitter for positional audio.
pub fn set_emitter(entity: EntityId, amp: f32) {
    wit::world_audio::set_emitter(entity.into_bindgen(), amp);
}

/// Plays a sound on the given entity.
pub fn play_sound_on_entity(url: String, amp: f32, emitter: EntityId) {
    wit::world_audio::set_emitter(emitter.into_bindgen(), amp);
    wit::world_audio::play_sound_on_entity(&url, emitter.into_bindgen());
}
