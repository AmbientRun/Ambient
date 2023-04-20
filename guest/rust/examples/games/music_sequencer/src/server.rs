use ambient_api::prelude::*;
use components::{note_selection, cursor, next_player_hue};

use ambient_api::{
    components::core::{
        player::{player, user_id},
    },
    entity::resources,
};

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


#[main]
pub async fn main() {
    // Set the initial next player hue.
    entity::add_component(resources(), next_player_hue(), 0.);

    let mut value = 0_u8;
    let mother = Entity::new()
    .with(note_selection(), vec![vec4(0.2, 0.2, 0.2, 1.); 128])
    .with(cursor(), 0)
    .spawn();

    let mut color_map = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
    let mut color_map_clone = color_map.clone();

    // When a player spawns, create their player state.
    spawn_query(user_id())
    .requires(player())
    .bind(move |players| {
        for (player, player_user_id) in players {
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

    messages::Click::subscribe(move |source, data| {
        let id = source.client_user_id().unwrap();
        let color_lock = color_map_clone.lock().unwrap();
        let color_to_set = color_lock.get(&id).unwrap();
        let mut v = entity::get_component(mother, note_selection()).unwrap();
        if v[data.index as usize] == *color_to_set {
            v[data.index as usize] = vec4(0.2, 0.2, 0.2, 1.);
        } else {
            v[data.index as usize] = *color_to_set;
        }
        entity::set_component(mother, note_selection(), v);
    });

    loop {
        value = (value + 1) % 16;
        entity::set_component(mother, cursor(), value);
        let v = entity::get_component(mother, note_selection()).unwrap();
        for i in 0..8 {
            let index = value as usize + i * 16;
            if v[index] != vec4(0.2, 0.2, 0.2, 1.) {
                messages::Play::new(i as u8).send_client_broadcast_reliable();
            }
        }
        sleep(0.125).await;
    }
}
