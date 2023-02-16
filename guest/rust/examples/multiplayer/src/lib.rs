use kiwi_api::{
    components::core::{
        camera::{aspect_ratio_from_window, perspective_infinite_reverse},
        game_objects::player_camera,
        player::player,
        primitives::cube,
        rendering::color,
        transform::{lookat_center, scale, translation},
    },
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    const CAMERA_POSITION: Vec3 = vec3(CUBE_REGION, CUBE_REGION, CUBE_REGION);
    const CUBE_REGION: f32 = 5.;
    const CUBE_SIZE: f32 = 0.6;

    entity::game_object_base()
        .with_default(player_camera())
        .with(translation(), CAMERA_POSITION)
        .with(lookat_center(), vec3(0., 0., 0.))
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio_from_window(), ())
        .spawn();

    spawn_query(player()).bind(move |players| {
        // For each player joining, spawn a random colored box somewhere
        for _ in players {
            let cube_position = rand::random::<Vec3>() * CUBE_REGION - vec3(0., 2. * CUBE_SIZE, 0.);
            entity::game_object_base()
                .with_default(cube())
                .with(scale(), Vec3::ONE * CUBE_SIZE)
                .with(translation(), cube_position)
                .with(color(), rand::random::<Vec3>().extend(1.))
                .spawn();

            println!("Cube created at {cube_position}");
        }
    });

    EventOk
}
