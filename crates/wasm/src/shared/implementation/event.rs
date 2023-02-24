use crate::shared::host_guest_state::BaseHostGuestState;
use ambient_ecs::EntityData;
use ambient_ecs::{world_events, WorldEvent};

pub fn subscribe(shared_state: &mut BaseHostGuestState, name: &str) {
    shared_state.subscribed_events.insert(name.to_string());
}

pub fn send(shared_state: &mut BaseHostGuestState, name: &str, data: EntityData) {
    if name.starts_with("core/") {
        return;
    }
    shared_state
        .world_mut()
        .resource_mut(world_events())
        .add_event(WorldEvent {
            name: name.to_string(),
            data,
        });
}
