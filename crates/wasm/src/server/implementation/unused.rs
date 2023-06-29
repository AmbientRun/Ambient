//! Used to stub out all the unused host functions on the serverside.
use crate::shared::{implementation::unsupported, wit};

use super::Bindings;

#[async_trait::async_trait]
impl wit::client_message::Host for Bindings {
    async fn send(
        &mut self,
        _: wit::client_message::Target,
        _: String,
        _: Vec<u8>,
    ) -> anyhow::Result<()> {
        unsupported()
    }
}
#[async_trait::async_trait]
impl wit::client_player::Host for Bindings {
    async fn get_local(&mut self) -> anyhow::Result<wit::types::EntityId> {
        unsupported()
    }
}
#[async_trait::async_trait]
impl wit::client_input::Host for Bindings {
    async fn get(&mut self) -> anyhow::Result<wit::client_input::Input> {
        unsupported()
    }
    async fn get_previous(&mut self) -> anyhow::Result<wit::client_input::Input> {
        unsupported()
    }
    async fn set_cursor(&mut self, _: wit::client_input::CursorIcon) -> anyhow::Result<()> {
        unsupported()
    }
    async fn set_cursor_visible(&mut self, _: bool) -> anyhow::Result<()> {
        unsupported()
    }
    async fn set_cursor_lock(&mut self, _: bool) -> anyhow::Result<()> {
        unsupported()
    }
}
#[async_trait::async_trait]
impl wit::client_camera::Host for Bindings {
    async fn clip_position_to_world_ray(
        &mut self,
        _camera: wit::types::EntityId,
        _clip_space_pos: wit::types::Vec2,
    ) -> anyhow::Result<wit::types::Ray> {
        unsupported()
    }
    async fn screen_to_clip_space(
        &mut self,
        _screen_pos: wit::types::Vec2,
    ) -> anyhow::Result<wit::types::Vec2> {
        unsupported()
    }

    async fn screen_position_to_world_ray(
        &mut self,
        _camera: wit::types::EntityId,
        _screen_pos: wit::types::Vec2,
    ) -> anyhow::Result<wit::types::Ray> {
        unsupported()
    }

    async fn world_to_screen(
        &mut self,
        _camera: wit::types::EntityId,
        _world_pos: wit::types::Vec3,
    ) -> anyhow::Result<wit::types::Vec2> {
        unsupported()
    }
}

#[async_trait::async_trait]
impl wit::client_window::Host for Bindings {
    async fn set_fullscreen(&mut self, _fullscreen: bool) -> anyhow::Result<()> {
        unsupported()
    }
}
#[async_trait::async_trait]
impl wit::client_mesh::Host for Bindings {
    async fn create(
        &mut self,
        _desc: wit::client_mesh::Descriptor,
    ) -> anyhow::Result<wit::client_mesh::Handle> {
        unsupported()
    }
    async fn destroy(&mut self, _handle: wit::client_mesh::Handle) -> anyhow::Result<()> {
        unsupported()
    }
}
#[async_trait::async_trait]
impl wit::client_texture::Host for Bindings {
    async fn create2d(
        &mut self,
        _desc: wit::client_texture::Descriptor2d,
    ) -> anyhow::Result<wit::client_texture::Handle> {
        unsupported()
    }
    async fn destroy(&mut self, _handle: wit::client_texture::Handle) -> anyhow::Result<()> {
        unsupported()
    }
}
#[async_trait::async_trait]
impl wit::client_sampler::Host for Bindings {
    async fn create(
        &mut self,
        _desc: wit::client_sampler::Descriptor,
    ) -> anyhow::Result<wit::client_sampler::Handle> {
        unsupported()
    }
    async fn destroy(&mut self, _handle: wit::client_sampler::Handle) -> anyhow::Result<()> {
        unsupported()
    }
}
#[async_trait::async_trait]
impl wit::client_material::Host for Bindings {
    async fn create(
        &mut self,
        _desc: wit::client_material::Descriptor,
    ) -> anyhow::Result<wit::client_material::Handle> {
        unsupported()
    }
    async fn destroy(&mut self, _handle: wit::client_material::Handle) -> anyhow::Result<()> {
        unsupported()
    }
}
