//! Used to implement all the *shared* host functions on the server.
//!
//! If implementing a trait that is only available on the server, it should go in [specific].
use crate::shared::{self, wit};

use super::Bindings;

mod specific;
mod unused;

impl wit::types::Host for Bindings {}
#[async_trait::async_trait]
impl wit::entity::Host for Bindings {
    async fn spawn(
        &mut self,
        data: wit::entity::EntityData,
    ) -> anyhow::Result<wit::types::EntityId> {
        shared::implementation::entity::spawn(
            unsafe { self.world_ref.world_mut() },
            &mut self.base.spawned_entities,
            data,
        )
    }

    async fn despawn(
        &mut self,
        entity: wit::types::EntityId,
    ) -> anyhow::Result<Option<wit::entity::EntityData>> {
        shared::implementation::entity::despawn(
            unsafe { self.world_ref.world_mut() },
            &mut self.base.spawned_entities,
            entity,
        )
    }

    async fn get_transforms_relative_to(
        &mut self,
        list: Vec<wit::types::EntityId>,
        origin: wit::types::EntityId,
    ) -> anyhow::Result<Vec<wit::types::Mat4>> {
        shared::implementation::entity::get_transforms_relative_to(self.world(), list, origin)
    }

    async fn exists(&mut self, entity: wit::types::EntityId) -> anyhow::Result<bool> {
        shared::implementation::entity::exists(self.world(), entity)
    }

    async fn resources(&mut self) -> anyhow::Result<wit::types::EntityId> {
        shared::implementation::entity::resources(self.world())
    }

    async fn synchronized_resources(&mut self) -> anyhow::Result<wit::types::EntityId> {
        shared::implementation::entity::synchronized_resources(self.world())
    }

    async fn persisted_resources(&mut self) -> anyhow::Result<wit::types::EntityId> {
        shared::implementation::entity::persisted_resources(self.world())
    }

    async fn in_area(
        &mut self,
        position: wit::types::Vec3,
        radius: f32,
    ) -> anyhow::Result<Vec<wit::types::EntityId>> {
        shared::implementation::entity::in_area(self.world_mut(), position, radius)
    }

    async fn get_all(&mut self, index: u32) -> anyhow::Result<Vec<wit::types::EntityId>> {
        shared::implementation::entity::get_all(self.world_mut(), index)
    }
}

#[async_trait::async_trait]
impl wit::component::Host for Bindings {
    async fn get_index(&mut self, id: String) -> anyhow::Result<Option<u32>> {
        shared::implementation::component::get_index(id)
    }

    async fn get_id(&mut self, index: u32) -> anyhow::Result<Option<String>> {
        shared::implementation::component::get_id(index)
    }

    async fn get_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
    ) -> anyhow::Result<Option<wit::component::Value>> {
        shared::implementation::component::get_component(self.world(), entity, index)
    }

    async fn get_components(
        &mut self,
        entity: wit::types::EntityId,
        indices: Vec<u32>,
    ) -> anyhow::Result<wit::entity::EntityData> {
        shared::implementation::component::get_components(self.world(), entity, indices)
    }

    async fn get_all_components(
        &mut self,
        entity: wit::types::EntityId,
    ) -> anyhow::Result<wit::entity::EntityData> {
        shared::implementation::component::get_all_components(self.world(), entity)
    }

    async fn add_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::Value,
    ) -> anyhow::Result<()> {
        shared::implementation::component::add_component(self.world_mut(), entity, index, value)
    }

    async fn add_components(
        &mut self,
        entity: wit::types::EntityId,
        data: wit::entity::EntityData,
    ) -> anyhow::Result<()> {
        shared::implementation::component::add_components(self.world_mut(), entity, data)
    }

    async fn set_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::Value,
    ) -> anyhow::Result<()> {
        shared::implementation::component::set_component(self.world_mut(), entity, index, value)
    }

    async fn set_components(
        &mut self,
        entity: wit::types::EntityId,
        data: wit::entity::EntityData,
    ) -> anyhow::Result<()> {
        shared::implementation::component::set_components(self.world_mut(), entity, data)
    }

    async fn has_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
    ) -> anyhow::Result<bool> {
        shared::implementation::component::has_component(self.world(), entity, index)
    }

    async fn has_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> anyhow::Result<bool> {
        shared::implementation::component::has_components(self.world(), entity, components)
    }

    async fn remove_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
    ) -> anyhow::Result<()> {
        shared::implementation::component::remove_component(self.world_mut(), entity, index)
    }

    async fn remove_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> anyhow::Result<()> {
        shared::implementation::component::remove_components(self.world_mut(), entity, components)
    }

    async fn query(
        &mut self,
        query: wit::component::QueryBuild,
        query_event: wit::component::QueryEvent,
    ) -> anyhow::Result<u64> {
        shared::implementation::component::query(&mut self.base.query_states, query, query_event)
    }

    async fn query_eval(
        &mut self,
        query_index: u64,
    ) -> anyhow::Result<Vec<(wit::types::EntityId, Vec<wit::component::Value>)>> {
        shared::implementation::component::query_eval(
            unsafe { self.world_ref.world() },
            &mut self.base.query_states,
            query_index,
        )
    }
}
#[async_trait::async_trait]
impl wit::message::Host for Bindings {
    async fn subscribe(&mut self, name: String) -> anyhow::Result<()> {
        shared::implementation::message::subscribe(&mut self.base.subscribed_messages, name)
    }
}
#[async_trait::async_trait]
impl wit::player::Host for Bindings {
    async fn get_by_user_id(
        &mut self,
        user_id: String,
    ) -> anyhow::Result<Option<wit::types::EntityId>> {
        shared::implementation::player::get_by_user_id(self.world(), user_id)
    }
}
#[async_trait::async_trait]
impl wit::asset::Host for Bindings {
    async fn url(
        &mut self,
        ember_id: String,
        path: String,
    ) -> anyhow::Result<Result<String, wit::asset::UrlError>> {
        shared::implementation::asset::url(self.world(), ember_id, path, false)
    }
}
