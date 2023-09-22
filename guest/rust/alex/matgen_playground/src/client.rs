use ambient_api::{
    core::{primitives::components::cube, transform::components::translation},
    prelude::*,
};

use packages::orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional};

#[main]
pub fn main() {
    OrbitCamera {
        optional: OrbitCameraOptional {
            camera_angle: Some(vec2(0.2, 0.2)),
            camera_distance: Some(5.0),
            lookat_target: Some(Vec3::ZERO),
        },
        ..OrbitCamera::suggested()
    }
    .make()
    .spawn();

    Entity::new()
        .with(cube(), ())
        .with(translation(), Vec3::ZERO)
        .spawn();
}
