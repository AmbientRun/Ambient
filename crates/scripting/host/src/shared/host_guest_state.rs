use std::collections::HashSet;

use elements_ecs::{EntityData, EntityUid, PrimitiveComponent, Query, QueryState};

pub type QueryStateMap =
    slotmap::SlotMap<slotmap::DefaultKey, (Query, QueryState, Vec<PrimitiveComponent>)>;

#[derive(Default, Clone)]
pub struct EventSharedState {
    pub subscribed_events: HashSet<String>,
    pub events: Vec<(String, EntityData)>,
}

#[derive(Default, Clone)]
pub struct BaseHostGuestState {
    pub spawned_entities: HashSet<EntityUid>,
    pub event: EventSharedState,
    pub query_states: QueryStateMap,
}

pub trait GetBaseHostGuestState {
    fn base_mut(&mut self) -> &mut BaseHostGuestState;
}
impl GetBaseHostGuestState for BaseHostGuestState {
    fn base_mut(&mut self) -> &mut BaseHostGuestState {
        self
    }
}
