use async_trait::async_trait;
use glam::{Mat4, Vec3};

pub use crate::internal::component::{
    query::{
        change_query, despawn_query, query, spawn_query, ChangeQuery, EventQuery, GeneralQuery,
        GeneralQueryBuilder, QueryEvent, UntrackedChangeQuery,
    },
    Component, ComponentOptionValue, ComponentValue, ComponentVecValue, ComponentsTuple, Entity,
    EnumComponent, SupportedValue, UntypedComponent, __internal_get_component,
};

#[doc(hidden)]
pub use crate::internal::wit::component::Value as WitComponentValue;
use crate::{
    core::ecs::components::{children, parent},
    prelude::{block_until, EntityId},
};

/// Implemented by all things that can be used as an ECS world.
pub trait World: Sync + Send {
    /// Spawns an entity containing the `components`.
    ///
    /// Returns `spawned_entity_uid`.
    fn spawn(&mut self, components: &Entity) -> EntityId;

    /// Despawns `entity` from the world. `entity` will not work with any other functions afterwards.
    ///
    /// Returns the data of the despawned entity, if it existed.
    fn despawn(&mut self, entity: EntityId) -> Option<Entity>;

    /// Gets a list of world transforms relative to origin entity
    /// Origin can be null entity for a list of world transforms
    fn get_transforms_relative_to(&self, list: &[EntityId], origin: EntityId) -> Vec<Mat4>;

    /// Checks if the `entity` exists.
    fn exists(&self, entity: EntityId) -> bool;

    /// Gets all of the entities that have the given `component`.
    #[doc(hidden)]
    fn get_all_untyped(&self, component: &dyn UntypedComponent) -> Vec<EntityId>;

    /// Gets all of the entities within `radius` of `position`.
    fn in_area(&self, position: Vec3, radius: f32) -> Vec<EntityId>;

    /// Retrieves the component `component` for `entity` if it exists, or `None` if it doesn't.
    #[doc(hidden)]
    fn get_component_untyped(
        &self,
        entity: EntityId,
        component: &dyn UntypedComponent,
    ) -> Option<ComponentValue>;

    /// Retrieves the components `components` for `entity`. Will return an empty `Entity` if no components are found.
    fn get_components(&self, entity: EntityId, components: &[&dyn UntypedComponent]) -> Entity;

    /// Retrieves all guest-visible components for `entity`. Will return an empty `Entity` if no components are found.
    ///
    /// Note that this may not be all of the components on the entity, as some components are not visible to the guest.
    fn get_all_components(&self, entity: EntityId) -> Entity;

    /// Adds the component `component` for `entity` with `value`. Will replace an existing component if present.
    #[doc(hidden)]
    fn add_component_untyped(
        &mut self,
        entity: EntityId,
        component: &dyn UntypedComponent,
        value: ComponentValue,
    );

    /// Adds the components `components` for `entity` with `value`. Will replace any existing components specified in `components`.
    fn add_components(&mut self, entity: EntityId, components: Entity);

    /// Sets the component `component` for `entity` with `value`.
    #[doc(hidden)]
    fn set_component_untyped(
        &mut self,
        entity: EntityId,
        component: &dyn UntypedComponent,
        value: ComponentValue,
    );

    /// Sets the components `components` for `entity` with `value`.
    fn set_components(&mut self, entity: EntityId, components: Entity);

    /// Checks if the `entity` has a `component`.
    #[doc(hidden)]
    fn has_component_untyped(&self, entity: EntityId, component: &dyn UntypedComponent) -> bool;

    /// Checks if the `entity` has `components`.
    fn has_components(&self, entity: EntityId, components: &[&dyn UntypedComponent]) -> bool;

    /// Removes the `component` from `entity`.
    ///
    /// Does nothing if the component does not exist.
    #[doc(hidden)]
    fn remove_component_untyped(&mut self, entity: EntityId, component: &dyn UntypedComponent);
    /// Removes the `components` from `entity`.
    ///
    /// Does nothing if the component does not exist.
    fn remove_components(&mut self, entity: EntityId, components: &[&dyn UntypedComponent]);

    /// Gets the resource entity. The components of this entity contain global state for this ECS world.
    ///
    /// Components with the `Resource` attribute can be found here.
    fn resources(&self) -> EntityId;

    /// Gets the synchronized resource entity. The components of this entity contain global state that should be networked, but not persisted.
    fn synchronized_resources(&self) -> EntityId;

    /// Gets the persisted resource entity. The components of this entity contain global state that should be networked and persisted.
    fn persisted_resources(&self) -> EntityId;
}

#[async_trait]
/// Extension methods for [`Worldlike`].
pub trait WorldExt: World {
    /// Gets all of the entities that have the given `component`.
    fn get_all<T: SupportedValue>(&self, component: Component<T>) -> Vec<EntityId> {
        self.get_all_untyped(&component)
    }

    /// Retrieves the component `component` for `entity` if it exists, or `None` if it doesn't.
    fn get_component<T: SupportedValue>(
        &self,
        entity: EntityId,
        component: Component<T>,
    ) -> Option<T> {
        self.get_component_untyped(entity, &component)
            .and_then(|x| T::from_value(x))
    }
    /// Retrieves the component `component` for `entity` if it exists, or `None` if it doesn't.
    fn get<T: SupportedValue>(&self, entity: EntityId, component: Component<T>) -> Option<T> {
        self.get_component(entity, component)
    }
    #[doc(hidden)]
    fn get_cloned<T: SupportedValue>(
        &self,
        entity: EntityId,
        component: Component<T>,
    ) -> Option<T> {
        self.get_component(entity, component)
    }

    /// Retrieves the `resource` if it exists and panics if it doesn't.
    fn resource<T: SupportedValue>(&self, component: Component<T>) -> T {
        self.get_component(self.resources(), component).unwrap()
    }

    /// Adds the component `component` for `entity` with `value`. Will replace an existing component if present.
    fn add_component<T: SupportedValue>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
        value: T,
    ) {
        self.add_component_untyped(entity, &component, value.into_value())
    }

    /// Sets the component `component` for `entity` with `value`.
    fn set_component<T: SupportedValue>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
        value: T,
    ) {
        self.set_component_untyped(entity, &component, value.into_value())
    }
    /// Sets the component `component` for `entity` with `value`.
    fn set<T: SupportedValue>(&mut self, entity: EntityId, component: Component<T>, value: T) {
        self.set_component(entity, component, value)
    }

    /// Checks if the `entity` has a `component`.
    fn has_component<T: SupportedValue>(&self, entity: EntityId, component: Component<T>) -> bool {
        self.has_component_untyped(entity, &component)
    }

    /// Removes the `component` from `entity`.
    ///
    /// Does nothing if the component does not exist.
    fn remove_component<T: SupportedValue>(&mut self, entity: EntityId, component: Component<T>) {
        self.remove_component_untyped(entity, &component)
    }

    /// Waits until `id` has the `component`. If the entity was deleted the method returns None.
    async fn wait_for_component<T: SupportedValue>(
        &self,
        entity: EntityId,
        component: Component<T>,
    ) -> Option<T> {
        block_until(move || !self.exists(entity) || self.has_component(entity, component)).await;
        self.get_component(entity, component)
    }

    /// Despawns `entity` and all of its children.
    fn despawn_recursive(&mut self, entity: EntityId) {
        if let Some(res) = self.despawn(entity) {
            if let Some(children) = res.get_ref(children()) {
                for c in children {
                    self.despawn_recursive(*c);
                }
            }
        }
    }

    /// Mutates the component `component` for `entity` using the passed in `mutator`, and returns its value.
    ///
    /// This will not set the component if the value is the same, which will prevent change events from
    /// being unnecessarily fired.
    fn mutate_component<T: SupportedValue + Clone + PartialEq>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
        mutator: impl FnOnce(&mut T),
    ) -> Option<T> {
        let mut value: T = self.get_component(entity, component)?;
        let orig_value = value.clone();
        mutator(&mut value);
        if value != orig_value {
            self.set_component(entity, component, value.clone());
        }
        Some(value)
    }

    /// Mutates the component `component` for `entity` using the passed in `mutator`, or sets it
    /// to `default` if it doesn't exist, and returns its value.
    ///
    /// This will not set the component if the value is the same, which will prevent change events from
    /// being unnecessarily fired.
    fn mutate_component_with_default<T: SupportedValue + Clone + PartialEq>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
        default: T,
        mutator: impl FnOnce(&mut T),
    ) -> T {
        let value = self.mutate_component(entity, component, mutator);
        if let Some(value) = value {
            value
        } else {
            self.add_component(entity, component, default.clone());
            default
        }
    }

    /// Adds `child` as a child to `entity`.
    fn add_child(&mut self, entity: EntityId, child: EntityId) {
        if self.has_component(entity, children()) {
            self.mutate_component(entity, children(), |children| children.push(child));
        } else {
            self.add_component(entity, children(), vec![child]);
        }
        self.add_component(child, parent(), entity);
    }

    /// Removes `child` as a child to `entity`.
    fn remove_child(&mut self, entity: EntityId, child: EntityId) {
        if self.has_component(entity, children()) {
            self.mutate_component(entity, children(), |children| {
                children.retain(|x| *x != child)
            });
        }
        self.remove_component(child, parent());
    }
}
impl<T: World + ?Sized> WorldExt for T {}
