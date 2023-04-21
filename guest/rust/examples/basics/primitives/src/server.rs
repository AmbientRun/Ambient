use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::{cube, quad, sphere_radius, sphere_sectors, sphere_stacks},
        rendering::color,
        transform::{lookat_target, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(5., 5., 6.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(cube())
        .with(translation(), vec3(0., 0., 1.))
        .with(scale(), Vec3::ONE * 2.)
        .with(color(), vec4(0., 1., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_sphere())
        .with(sphere_radius(), 1.)
        .with(sphere_sectors(), 12)
        .with(sphere_stacks(), 6)
        .with(translation(), vec3(0., 0., 3.))
        .with(color(), vec4(0., 0., 1., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_sphere())
        .with(translation(), vec3(0., 0., 4.5))
        .with(color(), vec4(1., 1., 0., 1.))
        .spawn();
}
