use ambient_api::prelude::*;
use packages::this::{assets, messages::Bonk};

#[main]
pub fn main() {
    let spatial_audio_player = audio::SpatialAudioPlayer::new();

    // println!("Hello, world!");
    Bonk::subscribe(move |_source, data| {
        spatial_audio_player.set_listener(data.listener);
        spatial_audio_player.play_sound_on_entity(assets::url("bonk.ogg"), data.emitter);
    });
}
