use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        player::components::is_player,
        primitives::components::cube,
        rendering::components::color,
        transform::{
            components::{lookat_target, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(main_scene(), ())
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    spawn_query(is_player()).bind(move |players| {
        // For each player joining, spawn a random colored box somewhere
        for _ in players {
            Entity::new()
                .with_merge(make_transformable())
                .with(cube(), ())
                .with(translation(), rand::random())
                .with(color(), rand::random::<Vec3>().extend(1.0))
                .spawn();
        }
    });
}
