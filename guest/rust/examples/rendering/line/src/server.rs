use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    core::{
        app::components::{main_scene, name, ui_scene},
        messages::Frame,
        primitives::{components::quad, concepts::Sphere},
        rect::components::{line_width, pixel_line_from, pixel_line_to},
        rendering::components::{cast_shadows, color, fog_density, light_diffuse, sky, sun, water},
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};
use packages::orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional};

#[main]
pub fn main() {
    main2();
}
fn main2() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            lookat_target: Some(Vec3::Z),
            camera_angle: Some(vec2(0., 20f32.to_radians())),
            camera_distance: Some(20.),
        },
    }
    .make()
    .spawn();

    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 20.)
        .spawn();

    Entity::new()
        .with(name(), "The line".to_string())
        .with(pixel_line_from(), vec3(0., 0., 1.))
        .with(pixel_line_to(), vec3(10., 0., 1.))
        .with(line_width(), 10.)
        .with(ui_scene(), ())
        .spawn();
}
