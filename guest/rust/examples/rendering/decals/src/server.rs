use ambient_api::{
    core::{
        primitives::components::{cube, quad},
        rendering::components::{color, decal_from_url, transparency_group},
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};
use packages::orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional};

use core::f32::consts::PI;

#[main]
pub fn main() {
    // Camera.
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            camera_angle: Some(vec2(135f32.to_radians(), 45f32.to_radians())),
            camera_distance: Some(2.),
            ..default()
        },
    }
    .spawn();

    // Scene geometry.
    Entity::new()
        .with(cube(), ())
        .with(translation(), vec3(-0.5, -0.5, 0.0))
        .with(scale(), vec3(0.9, 0.9, 0.9))
        .with(color(), vec4(0.5, 0.5, 0.5, 1.0))
        .spawn();
    Entity::new()
        .with(quad(), ())
        .with(scale(), 3.0 * Vec3::ONE)
        .spawn();

    // Decal projection volume.
    let decal_scale = vec3(1.0, 1.0, 1.0);
    let decal_rotation = Quat::from_rotation_z(PI / 4.0);
    let decal_url = packages::this::assets::url("pipeline.toml/0/mat.json");
    Entity::new()
        .with(rotation(), decal_rotation)
        .with(scale(), decal_scale)
        .with(decal_from_url(), decal_url)
        .spawn();

    // Decal projection volume visualization.
    Entity::new()
        .with(cube(), ())
        .with(rotation(), decal_rotation)
        .with(scale(), decal_scale)
        .with(color(), vec4(0.0, 1.0, 1.0, 0.5))
        .with(transparency_group(), 0)
        .spawn();
}
