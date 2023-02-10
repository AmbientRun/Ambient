use crate::{host, Components};

/// Fired each frame.
pub const FRAME: &str = "core/frame";
/// Fired on a collision. Components will contain the `ids` of the objects.
pub const COLLISION: &str = "core/collision";
/// Fired when a collider is loaded. Components will contain the `id` of the object.
pub const COLLIDER_LOAD: &str = "core/collider_load";
/// Fired when an entity is spawned. Components will contain the `id` and `uid` of the object.
pub const ENTITY_SPAWN: &str = "core/entity_spawn";
/// Fired when the module is loaded.
pub const MODULE_LOAD: &str = "core/module_load";
/// Fired when the module is unloaded.
pub const MODULE_UNLOAD: &str = "core/module_unload";

/// Sends an event
pub fn send(name: impl AsRef<str>, data: Components) {
    data.call_with(|data| host::event_send(name.as_ref(), data))
}
