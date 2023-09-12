use ambient_api::{
    core::{
        prefab::components::prefab_from_url,
        primitives::components::quad,
        rendering::components::cast_shadows,
        transform::components::{local_to_world, scale},
    },
    prelude::*,
};

use packages::{
    orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional},
    this::assets,
};

#[main]
pub fn main() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            lookat_target: Some(Vec3::Z * 1.5),
            camera_distance: Some(10.0),
            ..default()
        },
    }
    .spawn();

    Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 100.)
        .spawn();

    Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(cast_shadows(), ())
        .with(
            prefab_from_url(),
            assets::url("TreePack/Prefabs/Standard/Fir_01_Plant.prefab"),
        )
        .spawn();
}
