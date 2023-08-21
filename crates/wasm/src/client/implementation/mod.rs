//! Used to implement all the *shared* host functions on the client.
//!
//! If implementing a trait that is only available on the client, it should go in [specific].

use crate::shared::{self, wit};

use super::Bindings;

mod conversion;
mod specific;
mod unused;

impl wit::types::Host for Bindings {}

impl wit::entity::Host for Bindings {
    fn spawn(
        &mut self,
        data: wit::entity::EntityData,
    ) -> wasm_bridge::Result<wit::types::EntityId> {
        shared::implementation::entity::spawn(
            unsafe { self.world_ref.world_mut() },
            &mut self.base.spawned_entities,
            data,
        )
    }

    fn despawn(
        &mut self,
        entity: wit::types::EntityId,
    ) -> wasm_bridge::Result<Option<wit::entity::EntityData>> {
        shared::implementation::entity::despawn(
            unsafe { self.world_ref.world_mut() },
            &mut self.base.spawned_entities,
            entity,
        )
    }

    fn get_transforms_relative_to(
        &mut self,
        list: Vec<wit::types::EntityId>,
        origin: wit::types::EntityId,
    ) -> wasm_bridge::Result<Vec<wit::types::Mat4>> {
        shared::implementation::entity::get_transforms_relative_to(self.world(), list, origin)
    }

    fn exists(&mut self, entity: wit::types::EntityId) -> wasm_bridge::Result<bool> {
        shared::implementation::entity::exists(self.world(), entity)
    }

    fn resources(&mut self) -> wasm_bridge::Result<wit::types::EntityId> {
        shared::implementation::entity::resources(self.world())
    }

    fn synchronized_resources(&mut self) -> wasm_bridge::Result<wit::types::EntityId> {
        shared::implementation::entity::synchronized_resources(self.world())
    }

    fn persisted_resources(&mut self) -> wasm_bridge::Result<wit::types::EntityId> {
        shared::implementation::entity::persisted_resources(self.world())
    }

    fn in_area(
        &mut self,
        position: wit::types::Vec3,
        radius: f32,
    ) -> wasm_bridge::Result<Vec<wit::types::EntityId>> {
        shared::implementation::entity::in_area(self.world_mut(), position, radius)
    }

    fn get_all(&mut self, index: u32) -> wasm_bridge::Result<Vec<wit::types::EntityId>> {
        shared::implementation::entity::get_all(self.world_mut(), index)
    }
}

impl wit::component::Host for Bindings {
    fn get_index(&mut self, id: String) -> wasm_bridge::Result<Option<u32>> {
        shared::implementation::component::get_index(id)
    }

    fn get_id(&mut self, index: u32) -> wasm_bridge::Result<Option<String>> {
        shared::implementation::component::get_id(index)
    }

    fn get_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
    ) -> wasm_bridge::Result<Option<wit::component::Value>> {
        shared::implementation::component::get_component(self.world(), entity, index)
    }

    fn get_components(
        &mut self,
        entity: wit::types::EntityId,
        indices: Vec<u32>,
    ) -> wasm_bridge::Result<wit::entity::EntityData> {
        shared::implementation::component::get_components(self.world(), entity, indices)
    }

    fn get_all_components(
        &mut self,
        entity: wit::types::EntityId,
    ) -> wasm_bridge::Result<wit::entity::EntityData> {
        shared::implementation::component::get_all_components(self.world(), entity)
    }

    fn add_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::Value,
    ) -> wasm_bridge::Result<()> {
        shared::implementation::component::add_component(self.world_mut(), entity, index, value)
    }

    fn add_components(
        &mut self,
        entity: wit::types::EntityId,
        data: wit::entity::EntityData,
    ) -> wasm_bridge::Result<()> {
        shared::implementation::component::add_components(self.world_mut(), entity, data)
    }

    fn set_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::Value,
    ) -> wasm_bridge::Result<()> {
        shared::implementation::component::set_component(self.world_mut(), entity, index, value)
    }

    fn set_components(
        &mut self,
        entity: wit::types::EntityId,
        data: wit::entity::EntityData,
    ) -> wasm_bridge::Result<()> {
        shared::implementation::component::set_components(self.world_mut(), entity, data)
    }

    fn has_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
    ) -> wasm_bridge::Result<bool> {
        shared::implementation::component::has_component(self.world(), entity, index)
    }

    fn has_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> wasm_bridge::Result<bool> {
        shared::implementation::component::has_components(self.world(), entity, components)
    }

    fn remove_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
    ) -> wasm_bridge::Result<()> {
        shared::implementation::component::remove_component(self.world_mut(), entity, index)
    }

    fn remove_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> wasm_bridge::Result<()> {
        shared::implementation::component::remove_components(self.world_mut(), entity, components)
    }

    fn query(
        &mut self,
        query: wit::component::QueryBuild,
        query_event: wit::component::QueryEvent,
    ) -> wasm_bridge::Result<u64> {
        shared::implementation::component::query(&mut self.base.query_states, query, query_event)
    }

    fn query_eval(
        &mut self,
        query_index: u64,
    ) -> wasm_bridge::Result<Vec<(wit::types::EntityId, Vec<wit::component::Value>)>> {
        shared::implementation::component::query_eval(
            unsafe { self.world_ref.world() },
            &mut self.base.query_states,
            query_index,
        )
    }
}

impl wit::message::Host for Bindings {
    fn subscribe(&mut self, name: String) -> wasm_bridge::Result<()> {
        shared::implementation::message::subscribe(&mut self.base.subscribed_messages, name)
    }
}

impl wit::player::Host for Bindings {
    fn get_by_user_id(
        &mut self,
        user_id: String,
    ) -> wasm_bridge::Result<Option<wit::types::EntityId>> {
        shared::implementation::player::get_by_user_id(self.world(), user_id)
    }
}

impl wit::asset::Host for Bindings {
    fn url(&mut self, path: String) -> wasm_bridge::Result<Result<String, wit::asset::UrlError>> {
        shared::implementation::asset::url(self.world(), path, true)
    }
}
