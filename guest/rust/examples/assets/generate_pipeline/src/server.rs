use ambient_api::{
    core::{prefab::components::prefab_from_url, transform::components::local_to_world},
    prelude::*,
};

use packages::{
    orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional},
    this::assets,
};
#[main]
pub async fn main() {
    // Camera
    OrbitCamera {
        is_orbit_camera: (),
        lookat_target: Vec3::ZERO,
        optional: OrbitCameraOptional {
            camera_distance: Some(4.0),
            ..default()
        },
    }
    .make()
    .spawn();
    // Model
    Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(prefab_from_url(), assets::url("Cube.glb"))
        .spawn();
}
