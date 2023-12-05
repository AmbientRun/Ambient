use ambient_api::{
    core::{model::components::model_from_url, transform::components::translation},
    prelude::*,
};
use packages::{
    base_assets,
    character_animation::components::basic_character_animations,
    orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional},
};

pub mod packages;

#[main]
pub fn main() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            lookat_target: Some(vec3(0., 0., 1.)),
            camera_angle: Some(vec2(135f32.to_radians(), 45f32.to_radians())),
            camera_distance: Some(3.0),
        },
    }
    .spawn();

    for y in 0..10 {
        for x in 0..10 {
            Entity::new()
                .with(model_from_url(), base_assets::assets::url("Y Bot.fbx"))
                .with(basic_character_animations(), EntityId::null())
                .with(translation(), vec3(x as f32, y as f32, 0.))
                .spawn();
        }
    }
}
