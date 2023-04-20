use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::quad,
        transform::{lookat_target, translation},
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
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .spawn();

    println!("Hello, Ambient!");
}
