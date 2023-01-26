use crate::{entity, internal_get_component, lazy_component, EntityId, LazyComponent};

static PLAYER: LazyComponent<()> = lazy_component!("core::player::player");
static USER_ID: LazyComponent<String> = lazy_component!("core::player::user_id");

/// Gets all of the players currently on the server within this world.
///
/// This may not include all players on the server.
pub fn get_all() -> Vec<EntityId> {
    entity::query(*PLAYER)
}

/// Gets `player_id`'s name.
pub fn get_name(player_id: EntityId) -> Option<String> {
    entity::get_component(player_id, *USER_ID)
}
