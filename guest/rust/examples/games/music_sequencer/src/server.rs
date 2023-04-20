use ambient_api::prelude::*;
use common::NOTE_COUNT;
use components::{note_selection, next_player_hue};

use ambient_api::{
    components::core::{
        player::{player, user_id},
    },
    entity::resources,
};

mod common;

#[main]
pub async fn main() {

    entity::add_component(resources(), next_player_hue(), 0.);

    let color_map = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
    let color_map_clone = color_map.clone();

    // When a player spawns, create their player state.
    spawn_query(user_id())
    .requires(player())
    .bind(move |players| {
        for (_player, player_user_id) in players {
            let next_color = hsv_to_rgb(&[
                entity::get_component(resources(), next_player_hue()).unwrap_or_default(),
                0.7,
                1.0,
            ])
            .extend(1.);
            entity::mutate_component(resources(), next_player_hue(), |h| *h += 102.5);
            color_map.lock().unwrap().insert(player_user_id, next_color);
        };
    });

    entity::add_component(
        entity::synchronized_resources(),
        note_selection(),
        vec![vec4(0.2, 0.2, 0.2, 1.0); NOTE_COUNT],
    );

    messages::Click::subscribe(move |source, data| {
        let id = source.client_user_id().unwrap();
        let color_lock = color_map_clone.lock().unwrap();
        let color_to_set = color_lock.get(&id).unwrap();
        entity::mutate_component(
            entity::synchronized_resources(),
            note_selection(),
            |selection| {
                if selection[data.index as usize] == *color_to_set {
                    selection[data.index as usize] = vec4(0.2, 0.2, 0.2, 1.);
                } else {
                    selection[data.index as usize] = *color_to_set;
                }
            },
        );
    });
}

pub fn hsv_to_rgb([h, s, v]: &[f32; 3]) -> Vec3 {
    let c = v * s;
    let p = (h / 60.) % 6.;
    let x = c * (1.0 - ((p % 2.) - 1.).abs());
    let m = Vec3::ONE * (v - c);

    m + match p.trunc() as i32 {
        0 => vec3(c, x, 0.),
        1 => vec3(x, c, 0.),
        2 => vec3(0., c, x),
        3 => vec3(0., x, c),
        4 => vec3(x, 0., c),
        5 => vec3(c, 0., x),
        _ => vec3(0., 0., 0.),
    }
}