pub use ambient_api_core as api;
pub use api::{components::core as components, concepts, message::*, messages};

use std::future::Future;
pub fn run_async(_world: &ecs::World, future: impl Future<Output = ()> + Send + 'static) {
    api::prelude::run_async(async {
        future.await;
        api::prelude::OkEmpty
    });
}

/// Execute a future to completion on a worker thread.
///
/// This permits spawning thread local futures
pub fn run_async_local<F>(_world: &ecs::World, create: impl 'static + Send + FnOnce() -> F)
where
    F: 'static + Future,
    F::Output: Send + 'static,
{
    api::prelude::run_async(async {
        let future = create();
        future.await;
        api::prelude::OkEmpty
    });
}

pub async fn sleep(seconds: f32) {
    api::prelude::sleep(seconds).await;
}

pub mod ecs {
    use super::api;
    pub use api::{
        ecs::{Component, SupportedValue as ComponentValue, UntypedComponent},
        prelude::{Entity, EntityId},
    };

    #[derive(Clone, Copy)]
    pub struct World;
    impl World {
        pub fn spawn(&self, entity: Entity) -> EntityId {
            api::entity::spawn(&entity)
        }
        pub fn despawn(&self, entity_id: EntityId) -> Option<Entity> {
            api::entity::despawn(entity_id)
        }
        pub fn exists(&self, entity_id: EntityId) -> bool {
            api::entity::exists(entity_id)
        }
        pub fn set<T: ComponentValue>(
            &self,
            entity_id: EntityId,
            component: Component<T>,
            value: T,
        ) -> Result<(), ECSError> {
            // TODO: set_component needs to return errors
            api::entity::set_component(entity_id, component, value);
            Ok(())
        }
        pub fn add_component<T: ComponentValue>(
            &self,
            entity_id: EntityId,
            component: Component<T>,
            value: T,
        ) -> Result<(), ECSError> {
            // TODO: add_component needs to return errors
            api::entity::add_component(entity_id, component, value);
            Ok(())
        }
        pub fn add_components(
            &self,
            entity_id: EntityId,
            components: Entity,
        ) -> Result<(), ECSError> {
            // TODO: add_components needs to return errors
            api::entity::add_components(entity_id, components);
            Ok(())
        }
        pub fn get<T: ComponentValue>(
            &self,
            entity_id: EntityId,
            component: Component<T>,
        ) -> Result<T, ECSError> {
            api::entity::get_component(entity_id, component)
                .ok_or_else(|| ECSError::EntityDoesntHaveComponent)
        }
        // TODO: This should actually return &T
        pub fn get_ref<T: ComponentValue>(
            &self,
            entity_id: EntityId,
            component: Component<T>,
        ) -> Result<T, ECSError> {
            self.get(entity_id, component)
        }
        pub fn has_component<T: ComponentValue>(
            &self,
            entity_id: EntityId,
            component: Component<T>,
        ) -> bool {
            api::entity::has_component(entity_id, component)
        }
        pub fn resource<T: ComponentValue>(&self, component: Component<T>) -> T {
            api::entity::get_component(api::entity::resources(), component).unwrap()
        }
    }
    #[derive(Debug)]
    pub enum ECSError {
        EntityDoesntHaveComponent,
        NoSuchEntity,
    }

    pub struct ComponentDesc(Box<dyn UntypedComponent>);
    impl ComponentDesc {
        pub fn index(&self) -> u32 {
            self.0.index()
        }
    }
    impl<T: 'static> From<Component<T>> for ComponentDesc {
        fn from(value: Component<T>) -> Self {
            Self(Box::new(value))
        }
    }
}

pub mod window {
    use ambient_shared_types::CursorIcon;

    pub fn set_cursor(_world: &crate::ecs::World, cursor: CursorIcon) {
        #[cfg(feature = "client")]
        super::api::client::input::set_cursor(cursor);
        #[cfg(not(feature = "client"))]
        let _ = cursor;
    }

    pub async fn get_clipboard() -> Option<String> {
        None
    }

    pub async fn set_clipboard(_text: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Clipboard is not yet supported"))
    }
}
