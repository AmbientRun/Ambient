use kiwi_api::components::core::{
    app::main_scene,
    camera::{active_camera, aspect_ratio_from_window, perspective_infinite_reverse},
    primitives::cube,
    rendering::{color, outline},
    transform::{lookat_center, scale, translation},
};
use kiwi_api::{player::KeyCode, prelude::*};
use palette::{FromColor, Hsl, Srgb};

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
            let id = entity::game_object_base()
                .with_default(cube())
                .with(translation(), vec3(x as f32, y as f32, 0.))
                .with(scale(), vec3(0.3, 0.3, 0.3))
                .with(color(), vec4(0.1, 0.1, 0.1, 1.))
                .spawn(false);
            cells.push(id);
        }
    }

    on(event::FRAME, move |_| {
        for cell in &cells {
            entity::remove_component(*cell, outline());
        }
        let n_players = player::get_all().len();
        for (i, player) in player::get_all().into_iter().enumerate() {
            let player_color = Srgb::from_color(Hsl::from_components((
                360. * i as f32 / n_players as f32,
                1.,
                0.5,
            )));
            let player_color = vec4(player_color.red, player_color.green, player_color.blue, 1.);
            let cell = entity::get_component(player, components::cell()).unwrap_or_default();
            let Some((delta, _)) = player::get_raw_input_delta(player) else { continue; };

            let mut x = cell % 3;
            let mut y = cell / 3;

            let keys = &delta.keys;
            if keys.contains(&KeyCode::Left) || keys.contains(&KeyCode::A) {
                x = (x + 3 - 1) % 3;
            }
            if keys.contains(&KeyCode::Right) || keys.contains(&KeyCode::D) {
                x = (x + 1) % 3;
            }
            if keys.contains(&KeyCode::Up) || keys.contains(&KeyCode::W) {
                y = (y + 3 - 1) % 3;
            }
            if keys.contains(&KeyCode::Down) || keys.contains(&KeyCode::S) {
                y = (y + 1) % 3;
            }
            let cell = y * 3 + x;
            entity::add_component_if_required(cells[cell as usize], outline(), player_color);
            entity::set_component(player, components::cell(), cell);

            if delta.keys.contains(&KeyCode::Space) {
                entity::set_component(cells[cell as usize], color(), player_color);
            }
        }
        EventOk
    });

    EventOk
}
