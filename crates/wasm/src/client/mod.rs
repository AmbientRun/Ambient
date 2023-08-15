use crate::shared;
use ambient_ecs::{EntityId, SystemGroup, World};
use std::sync::Arc;

mod implementation;
mod network;

pub fn initialize(
    world: &mut World,
    messenger: Arc<dyn Fn(&World, EntityId, shared::MessageType, &str) + Send + Sync>,
) -> anyhow::Result<()> {
    // anyhow::bail!("test");
    shared::initialize(
        world,
        messenger,
        |id| Bindings {
            base: Default::default(),
            world_ref: Default::default(),
            id,
        },
        None,
    )?;

    network::initialize(world);

    Ok(())
}
pub fn systems() -> SystemGroup {
    SystemGroup::new("core/wasm/client", vec![Box::new(shared::systems())])
}

#[derive(Clone)]
struct Bindings {
    base: shared::bindings::BindingsBase,
    world_ref: shared::bindings::WorldRef,
    id: EntityId,
}
impl Bindings {
    pub fn world(&self) -> &World {
        unsafe { self.world_ref.world() }
    }
    pub fn world_mut(&mut self) -> &mut World {
        unsafe { self.world_ref.world_mut() }
    }
}

impl shared::bindings::BindingsBound for Bindings {
    fn base(&self) -> &shared::bindings::BindingsBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut shared::bindings::BindingsBase {
        &mut self.base
    }
    fn set_world(&mut self, world: &mut World) {
        unsafe {
            self.world_ref.set_world(world);
        }
    }
    fn clear_world(&mut self) {
        unsafe {
            self.world_ref.clear_world();
        }
    }
}
