use ambient_api::prelude::*;

#[main]
pub fn main() {
    let spatial_audio_player = audio::SpatialAudioPlayer::new();
    messages::Bonk::subscribe(move |_source, data| {
        spatial_audio_player.set_listener(data.listener);
        spatial_audio_player
            .play_sound_on_entity(asset::url("assets/bonk.ogg").unwrap(), data.emitter);
    });
}
