use crate::{components, entity, EntityId};

/// Gets all of the players currently on the server within this world.
///
/// This may not include all players on the server.
pub fn get_all() -> Vec<EntityId> {
    entity::query(components::core::player::player())
}

/// Gets `player_id`'s name.
pub fn get_name(player_id: EntityId) -> Option<String> {
    entity::get_component(player_id, components::core::player::user_id())
}
