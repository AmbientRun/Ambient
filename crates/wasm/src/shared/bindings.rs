use std::collections::HashSet;

use ambient_ecs::{EntityId, PrimitiveComponent, Query, QueryState, World};

pub type QueryStateMap =
    slotmap::SlotMap<slotmap::DefaultKey, (Query, QueryState, Vec<PrimitiveComponent>)>;

#[derive(Clone, Default)]
pub struct BindingsBase {
    pub spawned_entities: HashSet<EntityId>,
    pub subscribed_messages: HashSet<String>,
    pub query_states: QueryStateMap,
}

/// Represents all the bindings for the imported world
pub trait BindingsBound:
    // Lifetimes
    'static
    // Shared
    + super::wit::types::Host
    + super::wit::asset::Host
    + super::wit::component::Host
    + super::wit::entity::Host
    + super::wit::message::Host
    + super::wit::player::Host
    // Client
    + super::wit::client_message::Host
    + super::wit::client_player::Host
    + super::wit::client_input::Host
    + super::wit::client_camera::Host
    + super::wit::client_clipboard::Host
    + super::wit::client_window::Host
    + super::wit::client_mesh::Host
    + super::wit::client_texture::Host
    + super::wit::client_sampler::Host
    + super::wit::client_material::Host
    // Server
    + super::wit::server_asset::Host
    + super::wit::server_message::Host
    + super::wit::server_physics::Host
    + super::wit::server_http::Host
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

/// TODO: safety
unsafe impl Send for WorldRef {}
unsafe impl Sync for WorldRef {}
