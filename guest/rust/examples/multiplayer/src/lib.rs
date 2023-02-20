use kiwi_api::{
    components::core::{
        game_objects::player_camera,
        player::player,
        primitives::cube,
        rendering::color,
        transform::{lookat_center, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    make_perspective_infinite_reverse_camera()
        .with_default(player_camera())
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_center(), vec3(0., 0., 0.))
        .spawn();

    spawn_query(player()).bind(move |players| {
        // For each player joining, spawn a random colored box somewhere
        for _ in players {
            make_transformable()
                .with_default(cube())
                .with(translation(), rand::random())
                .with(color(), rand::random())
                .spawn();
        }
    });

    EventOk
}
