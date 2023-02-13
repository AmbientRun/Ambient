use std::collections::HashSet;

use kiwi_ecs::{EntityData, EntityUid, PrimitiveComponent, Query, QueryState, World};

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
    world_ref: WorldRef,
}
impl BaseHostGuestState {
    pub fn set_world(&mut self, world: &mut World) {
        self.world_ref.0 = world;
    }
    pub fn world(&self) -> &World {
        unsafe { self.world_ref.0.as_ref().unwrap() }
    }
    pub fn world_mut(&mut self) -> &mut World {
        unsafe { self.world_ref.0.as_mut().unwrap() }
    }
}

pub trait GetBaseHostGuestState {
    fn base(&self) -> &BaseHostGuestState;
    fn base_mut(&mut self) -> &mut BaseHostGuestState;
}
impl GetBaseHostGuestState for BaseHostGuestState {
    fn base(&self) -> &BaseHostGuestState {
        self
    }
    fn base_mut(&mut self) -> &mut BaseHostGuestState {
        self
    }
}

#[derive(Clone)]
struct WorldRef(pub *mut World);
impl Default for WorldRef {
    fn default() -> Self {
        Self::new()
    }
}
impl WorldRef {
    const fn new() -> Self {
        WorldRef(std::ptr::null_mut())
    }
}
unsafe impl Send for WorldRef {}
unsafe impl Sync for WorldRef {}
