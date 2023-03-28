use ambient_api::{
    components::core::{
        self,
        app::main_scene,
        camera::aspect_ratio_from_window,
        player::player,
        primitives::cube,
        rendering::{color, outline},
        transform::{lookat_center, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    player::KeyCode,
    prelude::*,
};
use components::cell;
use palette::{FromColor, Hsl, Srgb};

#[main]
pub fn main() -> ResultEmpty {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(3., 3., 2.5))
        .with(lookat_center(), vec3(1.5, 1.5, 0.))
        .spawn();

    let mut cells = Vec::new();
    for y in 0..3 {
        for x in 0..3 {
            let id = Entity::new()
                .with_merge(make_transformable())
                .with_default(cube())
                .with(translation(), vec3(x as f32, y as f32, 0.))
                .with(scale(), vec3(0.6, 0.6, 0.6))
                .with(color(), vec4(0.1, 0.1, 0.1, 1.))
                .spawn();
            cells.push(id);
        }
    }

    spawn_query(core::player::player()).bind(|ids| {
        for (id, _) in ids {
            entity::add_component(id, cell(), 0);
        }
    });

    on(event::FRAME, move |_| {
        for cell in &cells {
            entity::remove_component(*cell, outline());
        }

        let players = entity::get_all(player());
        let n_players = players.len();
        for (i, player) in players.into_iter().enumerate() {
            let player_color = Srgb::from_color(Hsl::from_components((
                360. * i as f32 / n_players as f32,
                1.,
                0.5,
            )));
            let player_color = vec4(player_color.red, player_color.green, player_color.blue, 1.);
            let Some(cell) = entity::get_component(player, components::cell()) else { continue; };
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
        OkEmpty
    });

    OkEmpty
}
