use components::core::{
    app::{main_scene, name}, camera::{
        active_camera, aspect_ratio, aspect_ratio_from_window, fovy, near, perspective_infinite_reverse, projection, projection_view
    }, ecs::dont_store, game_objects::player_camera, player::{player, user_id}, primitives::{cube, quad}, rendering::{color, outline, transparency_group}, transform::{inv_local_to_world, local_to_world, lookat_center, lookat_up, rotation, scale, translation}
};
use palette::{FromColor, Hsl, Srgb};
use tilt_runtime_scripting_interface::{
    player::{get_raw_input, KeyCode}, *
};

pub mod components;
pub mod params;

#[main]
pub async fn main() -> EventResult {
    entity::game_object_base()
        .with_default(main_scene())
        .with(active_camera(), 0.)
        .with(translation(), vec3(5.0, 5.0, 4.0))
        .with(lookat_center(), vec3(0., 0., 0.))
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio_from_window(), ())
        .spawn(false);

    let mut cells = Vec::new();
    for y in 0..3 {
        for x in 0..3 {
            let uid = entity::game_object_base()
                .with_default(cube())
                .with(translation(), vec3(x as f32, y as f32, 0.))
                .with(scale(), vec3(0.3, 0.3, 0.3))
                .with(color(), vec4(0.1, 0.1, 0.1, 1.))
                .spawn(false);
            cells.push(entity::wait_for_spawn(&uid).await);
        }
    }

    on(event::FRAME, move |_| {
        for cell in &cells {
            entity::remove_component(*cell, outline());
        }
        let n_players = player::get_all().len();
        for (i, player) in player::get_all().into_iter().enumerate() {
            let player_color = Srgb::from_color(Hsl::from_components((360. * i as f32 / n_players as f32, 1., 0.5)));
            let player_color = vec4(player_color.red, player_color.green, player_color.blue, 1.);
            // TODO: We're using transparency_group to store the state here, but should be done
            // with a custom component once that's implemented
            let cell = entity::get_component(player, transparency_group()).unwrap_or_default();
            let Some((delta, _)) = player::get_raw_input_delta(player) else { continue; };

            let mut x = cell % 3;
            let mut y = cell / 3;

            if delta.keys.contains(&KeyCode::Left) {
                x = (x + 3 - 1) % 3;
            }
            if delta.keys.contains(&KeyCode::Right) {
                x = (x + 1) % 3;
            }
            if delta.keys.contains(&KeyCode::Up) {
                y = (y + 3 - 1) % 3;
            }
            if delta.keys.contains(&KeyCode::Down) {
                y = (y + 1) % 3;
            }
            let cell = y * 3 + x;
            entity::add_component_if_required(cells[cell as usize], outline(), player_color);
            entity::set_component(player, transparency_group(), cell);

            if delta.keys.contains(&KeyCode::Space) {
                entity::set_component(cells[cell as usize], color(), player_color);
            }
        }
        EventOk
    });

    EventOk
}
