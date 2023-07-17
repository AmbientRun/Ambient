use ambient_api::prelude::*;

#[main]
pub fn main() {
    messages::StartGame::subscribe(|source, msg| {
        if let Some(id) = source.client_entity_id() {
            entity::add_component(id, components::player_name(), msg.name);
        }
    });
}
