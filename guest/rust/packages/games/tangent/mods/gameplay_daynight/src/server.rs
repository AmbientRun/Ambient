use ambient_api::{
    core::{rendering::components::sun, transform::components::rotation},
    prelude::*,
};

pub mod packages;

#[main]
pub fn main() {
    let start_game_time = game_time();

    query(sun()).each_frame(move |suns| {
        for (sun_id, _) in suns {
            let time = (game_time() - start_game_time).as_secs_f32();
            let yaw = (time * 4.0).to_radians();

            entity::add_component(
                sun_id,
                rotation(),
                Quat::from_rotation_y(-45_f32.to_radians()) * Quat::from_rotation_z(yaw),
            );
        }
    });
}
