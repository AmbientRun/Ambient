use ambient_api::{
    core::{
        physics::components::{dynamic, collider_from_url, kinematic, angular_velocity},
        transform::components::{translation, rotation},
        transform::concepts::make_transformable,
        prefab::components::prefab_from_url, model::components::model_from_url, messages::Collision,
    },
    prelude::*,
};
use packages::this::assets;

use crate::packages::tangent_schema::components::vehicle;

#[main]
pub async fn main() {
    let minigolf_map_id = make_transformable()
        .with(prefab_from_url(), assets::url("level.glb"))
        .with(translation(), Vec3::Z * -0.25)
        .spawn();
    let minigolf_fan_id = make_transformable()
        .with(model_from_url(), assets::url("fan.glb"))
        .with(collider_from_url(), assets::url("fan.glb"))
        .with(kinematic(), ())
        .with(dynamic(), true)
        .with(angular_velocity(), vec3(0., 90_f32.to_radians(), 0.))
        .with(translation(), vec3(-35., 161., 8.4331))
        .with(rotation(), Quat::from_rotation_z(180_f32.to_radians()))
        .spawn();

    Collision::subscribe(move |msg| {
        if msg.ids.iter().any(|id| *id == minigolf_map_id || *id == minigolf_fan_id) {
            for id in msg.ids {
                if entity::has_component(id, vehicle()) {
                    entity::set_component(id, model_from_url(), assets::url("ball.glb"));
                }
            }
        }
    });
}
