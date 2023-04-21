use ambient_api::prelude::*;
use common::NOTE_COUNT;
use components::{note_selection, next_player_hue, player_hue};

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

    // When a player spawns, create their player state.
    spawn_query(user_id())
    .requires(player())
    .bind(move |players| {
        for (player, _player_user_id) in players {
            let mut h = entity::get_component(resources(), next_player_hue()).unwrap();
            h = (h + 102.5) % 360.;
            entity::add_component(player, player_hue(), h);
            entity::set_component(resources(), next_player_hue(), h);
        };
    });

    entity::add_component(
        entity::synchronized_resources(),
        note_selection(),
        vec![0.0; NOTE_COUNT],
    );

    messages::Click::subscribe(move |source, data| {
        let id = source.client_entity_id().unwrap();
        let color_to_set = entity::get_component(id, player_hue()).unwrap();
        entity::mutate_component(
            entity::synchronized_resources(),
            note_selection(),
            |selection| {
                if selection[data.index as usize] == color_to_set {
                    selection[data.index as usize] = 0.0;
                } else {
                    selection[data.index as usize] = color_to_set;
                }
            },
        );
    });
}