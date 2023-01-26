use crate::{host, Components};

/// Triggered when the game server first starts
pub const GAME_START: &str = "core/game_start";
/// Triggered each frame
pub const FRAME: &str = "core/frame";
/// Triggered on a collision. Components will contain the `ids` of the objects
pub const COLLISION: &str = "core/collision";
/// Triggered when a collider is loaded. Components will contain the `id` of the object
pub const COLLIDER_LOAD: &str = "core/collider_load";
/// Triggered when an entity is spawned. Components will contain the `id` and `uid` of the object
pub const ENTITY_SPAWN: &str = "core/entity_spawn";

/// Sends an event
pub fn send(name: impl AsRef<str>, data: Components) {
    data.call_with(|data| host::event_send(name.as_ref(), data))
}
