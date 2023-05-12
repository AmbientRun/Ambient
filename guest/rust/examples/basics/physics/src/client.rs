use ambient_api::{components::core::physics::linear_velocity, prelude::*};

#[main]
pub fn main() {
    messages::Bonk::subscribe(move |_source, data| {
        spatial_audio::set_emitter(data.emitter);
        spatial_audio::set_listener(data.listener);
        spatial_audio::play_sound_on_entity(asset::url("assets/bonk.ogg").unwrap(), data.emitter);
    });
}
