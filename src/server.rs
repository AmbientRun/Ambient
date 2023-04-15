use std::f32::consts::PI;

use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        rendering::{fog_density, light_diffuse, sky, sun, water},
        transform::{lookat_center, rotation, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(5., 5., 2.))
        .with(lookat_center(), vec3(0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(water())
        .with(scale(), Vec3::ONE * 2000.)
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();

    let sun = Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with_default(rotation())
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.0)
        .spawn();

    ambient_api::messages::Frame::subscribe(move |_| {
        // How long a full cycle takes.
        const HALF_DAY_LENGTH: f32 = 30.0;

        entity::set_component(
            sun,
            rotation(),
            Quat::from_rotation_y(PI + PI * (time() * PI / HALF_DAY_LENGTH).sin().abs()),
        );
    });
}
