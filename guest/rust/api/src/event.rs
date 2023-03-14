use crate::{ecs::Entity, internal::wit};

pub use ambient_event_types::*;

/// Sends a (non-core) event to all other modules. This can be used for inter-module communication.
pub fn send(name: impl AsRef<str>, data: Entity) {
    data.call_with(|data| wit::event::send(name.as_ref(), data))
}
