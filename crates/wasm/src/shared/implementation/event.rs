use crate::shared::host_guest_state::BaseHostGuestState;
use ambient_core::name;
use ambient_ecs::world_events;
use ambient_ecs::Entity;

pub fn subscribe(shared_state: &mut BaseHostGuestState, name: &str) {
    shared_state.subscribed_events.insert(name.to_string());
}

pub fn send(shared_state: &mut BaseHostGuestState, event_name: &str, data: Entity) {
    if event_name.starts_with("core/") {
        return;
    }
    shared_state
        .world_mut()
        .resource_mut(world_events())
        .add_event(data.set(name(), event_name.to_string()));
}
