//! Used to stub out all the unused host functions on the clientside.

use super::Bindings;
use crate::shared::{implementation::unsupported, wit};
use wasm_bridge::async_trait;

#[async_trait]
impl wit::server_asset::Host for Bindings {
    async fn build_wasm(&mut self) -> anyhow::Result<()> {
        unsupported()
    }
}
#[async_trait]
impl wit::server_physics::Host for Bindings {
    async fn add_force(
        &mut self,
        _entity: wit::types::EntityId,
        _force: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        unsupported()
    }

    async fn add_impulse(
        &mut self,
        _entity: wit::types::EntityId,
        _force: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        unsupported()
    }

    async fn add_radial_impulse(
        &mut self,
        _position: wit::types::Vec3,
        _impulse: f32,
        _radius: f32,
        _falloff_radius: Option<f32>,
    ) -> anyhow::Result<()> {
        unsupported()
    }

    async fn add_force_at_position(
        &mut self,
        _entity: wit::types::EntityId,
        _force: wit::types::Vec3,
        _position: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        unsupported()
    }

    async fn add_impulse_at_position(
        &mut self,
        _entity: wit::types::EntityId,
        _force: wit::types::Vec3,
        _position: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        unsupported()
    }

    async fn get_velocity_at_position(
        &mut self,
        _entity: wit::types::EntityId,
        _position: wit::types::Vec3,
    ) -> anyhow::Result<wit::types::Vec3> {
        unsupported()
    }

    async fn set_gravity(&mut self, _gravity: wit::types::Vec3) -> anyhow::Result<()> {
        unsupported()
    }

    async fn unfreeze(&mut self, _entity: wit::types::EntityId) -> anyhow::Result<()> {
        unsupported()
    }

    async fn freeze(&mut self, _entity: wit::types::EntityId) -> anyhow::Result<()> {
        unsupported()
    }

    async fn start_motor(
        &mut self,
        _entity: wit::types::EntityId,
        _velocity: f32,
    ) -> anyhow::Result<()> {
        unsupported()
    }

    async fn stop_motor(&mut self, _entity: wit::types::EntityId) -> anyhow::Result<()> {
        unsupported()
    }

    async fn create_revolute_joint(
        &mut self,
        _entity0: wit::types::EntityId,
        _transform0: wit::types::Mat4,
        _entity1: wit::types::EntityId,
        _transform1: wit::types::Mat4,
    ) -> anyhow::Result<()> {
        unsupported()
    }

    async fn raycast_first(
        &mut self,
        _origin: wit::types::Vec3,
        _direction: wit::types::Vec3,
    ) -> anyhow::Result<Option<(wit::types::EntityId, f32)>> {
        unsupported()
    }

    async fn raycast(
        &mut self,
        _origin: wit::types::Vec3,
        _direction: wit::types::Vec3,
    ) -> anyhow::Result<Vec<(wit::types::EntityId, f32)>> {
        unsupported()
    }

    async fn move_character(
        &mut self,
        _entity: wit::types::EntityId,
        _displacement: wit::types::Vec3,
        _min_dist: f32,
        _elapsed_time: f32,
    ) -> anyhow::Result<wit::server_physics::CharacterCollision> {
        unsupported()
    }

    async fn set_character_position(
        &mut self,
        _entity: wit::types::EntityId,
        _position: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        unsupported()
    }

    async fn set_character_foot_position(
        &mut self,
        _entity: wit::types::EntityId,
        _position: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        unsupported()
    }
}
#[async_trait]
impl wit::server_message::Host for Bindings {
    async fn send(
        &mut self,
        _: wit::server_message::Target,
        _: String,
        _: Vec<u8>,
    ) -> anyhow::Result<()> {
        unimplemented!()
    }
}
#[async_trait]
impl wit::server_http::Host for Bindings {
    async fn get(&mut self, _: String) -> anyhow::Result<()> {
        unimplemented!()
    }
}
