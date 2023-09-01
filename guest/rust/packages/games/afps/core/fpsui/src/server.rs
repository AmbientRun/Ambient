use ambient_api::prelude::*;

use packages::afps_schema::{components::player_name, messages::StartGame};

#[main]
pub fn main() {
    StartGame::subscribe(|ctx, msg| {
        if let Some(id) = ctx.client_entity_id() {
            entity::add_component(id, player_name(), msg.name);
        }
    });
}
