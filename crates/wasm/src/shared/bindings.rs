use std::collections::HashSet;

use ambient_ecs::{EntityId, PrimitiveComponent, Query, QueryState, World};

use super::wit;

pub type QueryStateMap =
    slotmap::SlotMap<slotmap::DefaultKey, (Query, QueryState, Vec<PrimitiveComponent>)>;

#[derive(Clone, Default)]
pub struct BindingsBase {
    pub spawned_entities: HashSet<EntityId>,
    pub subscribed_messages: HashSet<String>,
    pub query_states: QueryStateMap,
}

pub trait BindingsBound:
    // Shared
    wit::types::Host
    + wit::asset::Host
    + wit::component::Host
    + wit::entity::Host
    + wit::message::Host
    + wit::player::Host
    // Client
    + wit::client_audio::Host
    + wit::client_message::Host
    + wit::client_player::Host
    + wit::client_input::Host
    + wit::client_camera::Host
    // Server
    + wit::server_message::Host
    + wit::server_physics::Host
    + Clone
    + Sync
    + Send
{
    fn base(&self) -> &BindingsBase;
    fn base_mut(&mut self) -> &mut BindingsBase;

    fn set_world(&mut self, world: &mut World);
    fn clear_world(&mut self);
}

#[derive(Clone)]
pub struct WorldRef(*mut World);
impl Default for WorldRef {
    fn default() -> Self {
        Self::new()
    }
}
impl WorldRef {
    const fn new() -> Self {
        WorldRef(std::ptr::null_mut())
    }
    pub unsafe fn world(&self) -> &World {
        unsafe { self.0.as_ref().unwrap() }
    }
    pub unsafe fn world_mut(&mut self) -> &mut World {
        unsafe { self.0.as_mut().unwrap() }
    }
    pub unsafe fn set_world(&mut self, world: &mut World) {
        self.0 = world;
    }
    pub unsafe fn clear_world(&mut self) {
        self.0 = std::ptr::null_mut();
    }
}
unsafe impl Send for WorldRef {}
unsafe impl Sync for WorldRef {}
