use ambient_api::{
    components::core::{
        app::main_scene,
        camera::{aspect_ratio_from_window},
        primitives::{cube, quad},
        rendering::color,
        transform::{lookat_target, rotation, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
fn main() {

    script::watch(asset::url("client.rhai").unwrap());
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(0., -5., 3.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 1000.)
        .with(color(), vec4(0., 0.5, 0.9, 1.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(cube())
        .with(scale(), Vec3::ONE * 1.)
        .spawn();
}