use ambient_api::{
    core::{ecs::components::remove_at_game_time, transform::components::translation},
    prelude::*,
};
use packages::this::messages::OnHealthPickup;

#[main]
pub fn main() {
    OnHealthPickup::subscribe(|ctx, msg| {
        if !ctx.server() {
            return;
        }

        let Some(active_camera) = camera::get_active(None) else {
            return;
        };

        // TODO: fix this once the audio API is revised
        let player = audio::SpatialAudioPlayer::new();
        entity::add_component(player.player, translation(), msg.position);
        entity::add_component(
            player.player,
            remove_at_game_time(),
            game_time() + Duration::from_secs(4),
        );
        player.set_listener(active_camera);
        player.play_sound_on_entity(
            packages::kenney_digital_audio::assets::url("powerUp2.ogg"),
            player.player,
        );
    });
}
