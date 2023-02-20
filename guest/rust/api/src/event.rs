use crate::{ecs::Components, internal::host};

/// Fired each frame.
pub const FRAME: &str = "core/frame";
/// Fired on a collision. Components will contain the `ids` of the objects.
pub const COLLISION: &str = "core/collision";
/// Fired when a collider is loaded. Components will contain the `id` of the object.
pub const COLLIDER_LOAD: &str = "core/collider_load";
/// Fired when an entity is spawned. Components will contain the `id` of the object.
pub const ENTITY_SPAWN: &str = "core/entity_spawn";
/// Fired when the module is loaded.
pub const MODULE_LOAD: &str = "core/module_load";
/// Fired when the module is unloaded.
pub const MODULE_UNLOAD: &str = "core/module_unload";

/// Sends a (non-core) event to all other modules. This can be used for inter-module communication.
pub fn send(name: impl AsRef<str>, data: Components) {
    data.call_with(|data| host::event_send(name.as_ref(), data))
}
