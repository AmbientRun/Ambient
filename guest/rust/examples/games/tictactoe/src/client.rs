use std::collections::HashMap;

use ambient_api::{
    components::core::rendering::{color, outline},
    message::{MessageExt, Target},
    player::KeyCode,
    prelude::*,
};
use palette::FromColor;

mod constants;

#[main]
async fn main() {
    entity::wait_for_component(entity::synchronized_resources(), components::cells()).await;

    let cells =
        entity::get_component(entity::synchronized_resources(), components::cells()).unwrap();

    ambient_api::messages::Frame::subscribe(move |_, _| {
        process_input();
        process_colors(&cells);
    });
}

fn process_input() {
    let (delta, _) = player::get_raw_input_delta();
    let keys = &delta.keys;
    let msg = messages::Input {
        left: keys.contains(&KeyCode::Left) || keys.contains(&KeyCode::A),
        right: keys.contains(&KeyCode::Right) || keys.contains(&KeyCode::D),
        up: keys.contains(&KeyCode::Up) || keys.contains(&KeyCode::W),
        down: keys.contains(&KeyCode::Down) || keys.contains(&KeyCode::S),
        capture: keys.contains(&KeyCode::Space),
    };

    if [msg.left, msg.right, msg.up, msg.down, msg.capture]
        .into_iter()
        .any(|x| x)
    {
        msg.send(Target::RemoteReliable);
    }
}

fn process_colors(cells: &[EntityId]) {
    for cell in cells {
        entity::remove_component(*cell, outline());
    }

    let players = entity::get_all(ambient_api::components::core::player::player());
    let n_players = players.len();
    let player_colors: HashMap<_, _> = players
        .iter()
        .enumerate()
        .map(|(i, player)| {
            let player_color = palette::Srgb::from_color(palette::Hsl::from_components((
                360. * i as f32 / n_players as f32,
                1.,
                0.5,
            )));
            let player_color = vec4(player_color.red, player_color.green, player_color.blue, 1.);
            (*player, player_color)
        })
        .collect();

    for (player, player_color) in player_colors.iter() {
        let Some(cell) = entity::get_component(*player, components::cell()) else { continue; };
        entity::add_component_if_required(cells[cell as usize], outline(), *player_color);
    }

    for cell in cells {
        let cell_color = entity::get_component(*cell, components::owned_by())
            .and_then(|id| player_colors.get(&id))
            .copied()
            .unwrap_or(constants::DEFAULT_COLOR);

        entity::set_component(*cell, color(), cell_color);
    }
}
