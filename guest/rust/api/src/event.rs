use crate::{ecs::Entity, internal::wit};

/// Fired each frame.
pub const FRAME: &str = "core/frame";
/// Fired on a collision. Components will contain the `ids` of the objects.
pub const COLLISION: &str = "core/collision";
/// Fired when a collider is loaded. Components will contain the `id` of the object.
pub const COLLIDER_LOAD: &str = "core/collider_load";
/// Fired when the module is loaded.
pub const MODULE_LOAD: &str = "core/module_load";
/// Fired when the module is unloaded.
pub const MODULE_UNLOAD: &str = "core/module_unload";
/// A world event was fired.
pub const WORLD_EVENT: &str = "core/world_event";

/// Sends a (non-core) event to all other modules. This can be used for inter-module communication.
pub fn send(name: impl AsRef<str>, data: Entity) {
    data.call_with(|data| wit::event::send(name.as_ref(), data))
}
