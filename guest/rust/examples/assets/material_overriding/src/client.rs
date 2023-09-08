use std::f32::consts::TAU;

use ambient_api::{
    core::{model::components::model_loaded, transform::components::rotation},
    glam::EulerRot,
    prelude::*,
};

#[main]
fn main() {
    // Load the asset
    println!(
        "asset url can be accessed from client: {}",
        packages::this::assets::url("Cube.glb")
    );

    query(model_loaded()).requires(rotation()).each_frame(|v| {
        let t = game_time().as_secs_f32();
        for (id, _) in v {
            entity::set_component(
                id,
                rotation(),
                Quat::from_euler(EulerRot::ZXY, t % TAU, (t * 2.0).sin() * 0.5, 0.0),
            );
        }
    });
}
