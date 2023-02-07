use components::core::{
    app::main_scene, camera::{active_camera, aspect_ratio_from_window, perspective_infinite_reverse}, player::{player, user_id}, primitives::cube, rendering::color, transform::{lookat_center, scale, translation}
};
use tilt_runtime_scripting_interface::*;

#[main]
pub async fn main() -> EventResult {
    const CAMERA_POSITION: Vec3 = vec3(CUBE_REGION, CUBE_REGION, CUBE_REGION);
    const CUBE_REGION: f32 = 5.0;
    const CUBE_SIZE: f32 = 0.3;

    entity::game_object_base()
        .with_default(main_scene())
        .with(active_camera(), 0.)
        .with(translation(), CAMERA_POSITION)
        .with(lookat_center(), vec3(0., 0., 0.))
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio_from_window(), ())
        .spawn(false);

    spawn_query(player()).bind(move |players| {
        // For each player joining, spawn a random colored box somewhere
        for _ in players {
            let cube_position = rand::random::<Vec3>() * CUBE_REGION - vec3(0.0, 2.0 * CUBE_SIZE, 0.0);
            entity::game_object_base()
                .with_default(cube())
                .with(scale(), Vec3::ONE * CUBE_SIZE)
                .with(translation(), cube_position)
                .with(color(), rand::random::<Vec3>().extend(1.))
                .spawn(false);

            println!("Cube created at {cube_position}");
        }
    });

    EventOk
}
