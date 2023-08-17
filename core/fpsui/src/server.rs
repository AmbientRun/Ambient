use ambient_api::prelude::*;

use packages::afps_schema::{components::player_name, messages::StartGame};

#[main]
pub fn main() {
    StartGame::subscribe(|source, msg| {
        if let Some(id) = source.client_entity_id() {
            entity::add_component(id, player_name(), msg.name);
        }
    });
}
