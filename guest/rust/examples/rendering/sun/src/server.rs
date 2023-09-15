use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    core::{
        app::components::main_scene,
        messages::Frame,
        primitives::{components::quad, concepts::Sphere},
        rendering::components::{cast_shadows, color, fog_density, light_diffuse, sky, sun, water},
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};
use packages::orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional};

#[main]
pub fn main() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            lookat_target: Some(Vec3::Z),
            camera_angle: Some(vec2(FRAC_PI_2, 20f32.to_radians())),
            camera_distance: None,
        },
    }
    .make()
    .spawn();

    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 20.)
        .with(color(), vec4(1., 0., 0., 1.))
        .with(translation(), vec3(0., 0., 0.01))
        .spawn();

    Entity::new()
        .with(water(), ())
        .with(scale(), Vec3::ONE * 2000.)
        .spawn();

    Entity::new().with(sky(), ()).spawn();

    Entity::new()
        .with_merge(Sphere {
            sphere: (),
            sphere_radius: 1.,
            sphere_sectors: 36,
            sphere_stacks: 18,
        })
        .with(cast_shadows(), ())
        .with(translation(), vec3(0., 0., 1.))
        .with(color(), vec4(1., 1., 1., 1.))
        .spawn();

    let sun = Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::IDENTITY)
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.)
        .spawn();

    Frame::subscribe(move |_| {
        let rot = entity::get_component(sun, rotation()).unwrap();
        entity::set_component(sun, rotation(), rot * Quat::from_rotation_y(0.01));
    });
}
