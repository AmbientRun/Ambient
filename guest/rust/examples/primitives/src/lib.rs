use kiwi_api::{
    components::core::{
        game_objects::player_camera,
        primitives::{cube, quad, sphere_radius, sphere_sectors, sphere_stacks},
        rendering::color,
        transform::{lookat_center, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    make_perspective_infinite_reverse_camera()
        .with_default(player_camera())
        .with(translation(), vec3(5., 5., 6.))
        .with(lookat_center(), vec3(0., 0., 2.))
        .spawn();

    make_transformable()
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    make_transformable()
        .with_default(cube())
        .with(translation(), vec3(0., 0., 1.))
        .with(scale(), Vec3::ONE * 2.)
        .with(color(), vec4(0., 1., 0., 1.))
        .spawn();

    make_sphere()
        .with(sphere_radius(), 1.)
        .with(sphere_sectors(), 12)
        .with(sphere_stacks(), 6)
        .with(translation(), vec3(0., 0., 3.))
        .with(color(), vec4(0., 0., 1., 1.))
        .spawn();

    make_sphere()
        .with(translation(), vec3(0., 0., 4.5))
        .with(color(), vec4(1., 1., 0., 1.))
        .spawn();

    EventOk
}
