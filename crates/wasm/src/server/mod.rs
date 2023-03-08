use crate::shared::{self, wit};
use ambient_ecs::{query, EntityId, FnSystem, SystemGroup, World};
use ambient_network::server::{ForkingEvent, ShutdownEvent};
use std::sync::Arc;

mod conversion;
mod implementation;

pub fn initialize(
    world: &mut World,
    messenger: Arc<dyn Fn(&World, EntityId, shared::MessageType, &str) + Send + Sync>,
) -> anyhow::Result<()> {
    shared::initialize(
        world,
        messenger,
        Bindings {
            base: Default::default(),
            world_ref: Default::default(),
        },
    )?;

    Ok(())
}

pub fn systems() -> SystemGroup {
    shared::systems()
}

pub fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    SystemGroup::new(
        "core/wasm/server/on_forking_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            // Reset the states of all the modules when we fork.
            shared::reload_all(world);
        }))],
    )
}

pub fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    SystemGroup::new(
        "core/wasm/server/on_shutdown_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            let modules = query(()).incl(shared::module()).collect_ids(world, None);
            for module_id in modules {
                let errors = shared::unload(world, module_id, "shutting down");
                shared::update_errors(world, &errors);
            }
        }))],
    )
}

#[derive(Clone)]
struct Bindings {
    base: shared::bindings::BindingsBase,
    world_ref: shared::bindings::WorldRef,
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

impl wit::types::Host for Bindings {}
impl wit::entity::Host for Bindings {
    fn spawn(&mut self, data: wit::entity::EntityData) -> anyhow::Result<wit::types::EntityId> {
        shared::implementation::entity::spawn(
            unsafe { self.world_ref.world_mut() },
            &mut self.base.spawned_entities,
            data,
        )
    }

    fn despawn(&mut self, entity: wit::types::EntityId) -> anyhow::Result<bool> {
        shared::implementation::entity::despawn(
            unsafe { self.world_ref.world_mut() },
            &mut self.base.spawned_entities,
            entity,
        )
    }

    fn set_animation_controller(
        &mut self,
        entity: wit::types::EntityId,
        animation_controller: wit::entity::AnimationController,
    ) -> anyhow::Result<()> {
        shared::implementation::entity::set_animation_controller(
            self.world_mut(),
            entity,
            animation_controller,
        )
    }

    fn exists(&mut self, entity: wit::types::EntityId) -> anyhow::Result<bool> {
        shared::implementation::entity::exists(self.world(), entity)
    }

    fn resources(&mut self) -> anyhow::Result<wit::types::EntityId> {
        shared::implementation::entity::resources(self.world())
    }

    fn in_area(
        &mut self,
        position: wit::types::Vec3,
        radius: f32,
    ) -> anyhow::Result<Vec<wit::types::EntityId>> {
        shared::implementation::entity::in_area(self.world_mut(), position, radius)
    }

    fn get_all(&mut self, index: u32) -> anyhow::Result<Vec<wit::types::EntityId>> {
        shared::implementation::entity::get_all(self.world_mut(), index)
    }
}
impl wit::component::Host for Bindings {
    fn get_index(&mut self, id: String) -> anyhow::Result<Option<u32>> {
        shared::implementation::component::get_index(id)
    }

    fn get_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
    ) -> anyhow::Result<Option<wit::component::ValueResult>> {
        shared::implementation::component::get_component(self.world(), entity, index)
    }

    fn add_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::ValueResult,
    ) -> anyhow::Result<()> {
        shared::implementation::component::add_component(self.world_mut(), entity, index, value)
    }

    fn add_components(
        &mut self,
        entity: wit::types::EntityId,
        data: wit::entity::EntityData,
    ) -> anyhow::Result<()> {
        shared::implementation::component::add_components(self.world_mut(), entity, data)
    }

    fn set_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::ValueResult,
    ) -> anyhow::Result<()> {
        shared::implementation::component::set_component(self.world_mut(), entity, index, value)
    }

    fn set_components(
        &mut self,
        entity: wit::types::EntityId,
        data: wit::entity::EntityData,
    ) -> anyhow::Result<()> {
        shared::implementation::component::set_components(self.world_mut(), entity, data)
    }

    fn has_component(&mut self, entity: wit::types::EntityId, index: u32) -> anyhow::Result<bool> {
        shared::implementation::component::has_component(self.world(), entity, index)
    }

    fn has_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> anyhow::Result<bool> {
        shared::implementation::component::has_components(self.world(), entity, components)
    }

    fn remove_component(&mut self, entity: wit::types::EntityId, index: u32) -> anyhow::Result<()> {
        shared::implementation::component::remove_component(self.world_mut(), entity, index)
    }

    fn remove_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> anyhow::Result<()> {
        shared::implementation::component::remove_components(self.world_mut(), entity, components)
    }

    fn query(
        &mut self,
        query: wit::component::QueryBuild,
        query_event: wit::component::QueryEvent,
    ) -> anyhow::Result<u64> {
        shared::implementation::component::query(&mut self.base.query_states, query, query_event)
    }

    fn query_eval(
        &mut self,
        query_index: u64,
    ) -> anyhow::Result<Vec<(wit::types::EntityId, Vec<wit::component::ValueResult>)>> {
        shared::implementation::component::query_eval(
            unsafe { self.world_ref.world() },
            &mut self.base.query_states,
            query_index,
        )
    }
}
impl wit::event::Host for Bindings {
    fn subscribe(&mut self, name: String) -> anyhow::Result<()> {
        shared::implementation::event::subscribe(&mut self.base.subscribed_events, name)
    }

    fn send(&mut self, name: String, data: wit::entity::EntityData) -> anyhow::Result<()> {
        shared::implementation::event::send(
            self.world_mut(),
            name,
            shared::implementation::component::convert_components_to_entity_data(data),
        )
    }
}
