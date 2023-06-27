use ambient_api::prelude::*;

#[main]
pub fn main() {
    messages::StartGame::subscribe(|_, msg| {
        entity::add_component(msg.id, components::player_name(), msg.name);
    });
}
