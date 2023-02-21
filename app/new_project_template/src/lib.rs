use ambient_api::{
    components::core::{
        game_objects::player_camera,
        primitives::quad,
        transform::{lookat_center, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with_default(player_camera())
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_center(), vec3(0., 0., 0.))
        .spawn();

    Entity::new().with_merge(make_transformable()).with_default(quad()).spawn();

    println!("Hello, Ambient!");

    EventOk
}
