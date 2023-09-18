use ambient_api::{core::transform::components::translation, prelude::*};
use packages::tangent_schema::weapon::messages::Fire;

#[main]
pub fn main() {
    Fire::subscribe(|ctx, msg| {
        if !ctx.server() {
            return;
        }

        let Some(translation) = entity::get_component(msg.weapon_id, translation()) else {
            return;
        };

        audio::SpatialAudioPlayer::oneshot(
            translation,
            packages::kenney_digital_audio::assets::url("laser4.ogg"),
        );
    });
}
