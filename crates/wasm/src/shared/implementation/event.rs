use std::collections::HashSet;

use ambient_core::name;
use ambient_ecs::{world_events, Entity, World};

pub fn subscribe(subscribed_events: &mut HashSet<String>, name: &str) {
    subscribed_events.insert(name.to_string());
}

pub fn send(world: &mut World, event_name: &str, data: Entity) {
    if event_name.starts_with("core/") {
        return;
    }
    world
        .resource_mut(world_events())
        .add_event(data.with(name(), event_name.to_string()));
}
