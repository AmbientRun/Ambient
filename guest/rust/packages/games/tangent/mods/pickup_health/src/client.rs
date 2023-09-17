use ambient_api::prelude::*;
use packages::this::messages::OnHealthPickup;

#[main]
pub fn main() {
    OnHealthPickup::subscribe(|ctx, msg| {
        if !ctx.server() {
            return;
        }

        audio::SpatialAudioPlayer::oneshot(
            msg.position,
            packages::kenney_digital_audio::assets::url("powerUp2.ogg"),
        );
    });
}
