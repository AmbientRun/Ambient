use crate::{
    global::EntityId,
    internal::{conversion::FromBindgen, wit},
};

/// Get a player's entity ID from their user ID.
pub fn get_by_user_id(user_id: &str) -> Option<EntityId> {
    wit::player::get_by_user_id(user_id).from_bindgen()
}

/// Get the local player's entity ID.
#[cfg(feature = "client")]
pub fn get_local() -> EntityId {
    wit::client_player::get_local().from_bindgen()
}
