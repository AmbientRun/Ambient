use std::collections::HashSet;

use ambient_core::name;
use ambient_ecs::{world_events, Entity, World};

pub fn subscribe(subscribed_events: &mut HashSet<String>, name: String) -> anyhow::Result<()> {
    subscribed_events.insert(name);
    Ok(())
}

pub fn send(world: &mut World, event_name: String, data: Entity) -> anyhow::Result<()> {
    if event_name.starts_with("core/") {
        return Ok(());
    }

    world
        .resource_mut(world_events())
        .add_event(data.with(name(), event_name));
    Ok(())
}
