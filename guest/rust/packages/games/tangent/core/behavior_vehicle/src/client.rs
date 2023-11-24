use ambient_api::{
    core::{physics::components::linear_velocity, transform::components::rotation},
    once_cell::sync::Lazy,
    prelude::*,
};

use packages::{
    tangent_schema::vehicle::{client::components::speed_kph, components::is_vehicle},
    this::messages::OnCollision,
};

pub mod packages;

#[main]
pub fn main() {
    static SOUNDS: Lazy<[Vec<String>; 3]> = Lazy::new(|| {
        let url = |ty, idx| {
            packages::kenney_impact_sounds::assets::url(&format!("impactPlate_{ty}_{idx:0>3}.ogg"))
        };

        ["light", "medium", "heavy"].map(|ty| (0..5).map(|idx| url(ty, idx)).collect())
    });

    query((rotation(), linear_velocity()))
        .requires(is_vehicle())
        .each_frame(|vehicles| {
            for (id, (rot, lv)) in vehicles {
                entity::add_component(id, speed_kph(), lv.dot(rot * -Vec3::Y) * 3.6);
            }
        });

    OnCollision::subscribe(|ctx, msg| {
        if !ctx.server() {
            return;
        }

        let impact_type = match msg.speed {
            speed if speed < 5. => 0,
            speed if speed < 10. => 1,
            _ => 2,
        };

        let sound = SOUNDS[impact_type].choose(&mut thread_rng()).unwrap();
        audio::SpatialAudioPlayer::oneshot(msg.position, sound);
    });
}
