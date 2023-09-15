use ambient_api::{
    core::{
        primitives::components::cube, rendering::components::color,
        transform::components::translation,
    },
    prelude::*,
};

use packages::{
    orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional},
    this::components::{grid_position, grid_side_length},
};

#[main]
pub async fn main() {
    let side_length =
        entity::wait_for_component(entity::synchronized_resources(), grid_side_length())
            .await
            .unwrap();

    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            camera_distance: Some(20.0),
            ..default()
        },
    }
    .spawn();

    let start_time = game_time();

    query(grid_position())
        .requires(cube())
        .each_frame(move |entities| {
            for (id, position) in entities {
                let [x, y] = position.to_array();
                let grid_cell = position - glam::ivec2(side_length, side_length);
                let t = (game_time() - start_time).as_secs_f32();
                entity::mutate_component(id, translation(), |v| {
                    v.z = (x as f32 + y as f32 + t).sin() - 0.5 * grid_cell.as_vec2().length();
                });

                let s = (t.sin() + 1.0) / 2.0;
                let t = (((x + y) as f32).sin() + 1.0) / 2.0;
                entity::set_component(id, color(), vec3(s, 1.0 - s, t).extend(1.0));
            }
        });
}
