use ambient_api::{
    core::transform::components::{local_to_world, translation},
    prelude::*,
};
use packages::this::messages::Fire;

#[main]
pub fn main() {
    Fire::subscribe(|ctx, msg| {
        if !ctx.server() {
            return;
        }

        let Some(local_to_world) = entity::get_component(msg.weapon_id, local_to_world()) else {
            return;
        };

        let (_, _, translation) = local_to_world.to_scale_rotation_translation();

        audio::SpatialAudioPlayer::oneshot(
            translation,
            packages::kenney_digital_audio::assets::url("laser4.ogg"),
        );
    });
}
