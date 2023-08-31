#![allow(clippy::single_match)]
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
    internal::HostWorld,
    prelude::{block_until, EntityId},
};

#[derive(Debug)]
/// An error that can occur when interacting with the ECS.
pub enum ECSError {
    /// The entity does not have the component.
    EntityDoesntHaveComponent,
    /// The entity does not exist.
    NoSuchEntity,
}

/// A result that can occur when interacting with the ECS.
pub type Result<T> = core::result::Result<T, ECSError>;

/// An ECS world.
pub enum World {
    #[doc(hidden)]
    Host(HostWorld),
}

impl World {
    /// Spawns an entity containing the `components`.
    ///
    /// Returns `spawned_entity_id`.
    pub fn spawn(&mut self, components: Entity) -> EntityId {
        match self {
            World::Host(w) => w.spawn(components),
        }
    }

    /// Despawns `entity` from the world. `entity` will not work with any other functions afterwards.
    ///
    /// Returns the data of the despawned entity, if it existed.
    pub fn despawn(&mut self, entity: EntityId) -> Option<Entity> {
        match self {
            World::Host(w) => w.despawn(entity),
        }
    }

    /// Gets a list of world transforms relative to origin entity
    /// Origin can be null entity for a list of world transforms
    pub fn get_transforms_relative_to(&self, list: &[EntityId], origin: EntityId) -> Vec<Mat4> {
        match self {
            World::Host(w) => w.get_transforms_relative_to(list, origin),
        }
    }

    /// Checks if the `entity` exists.
    pub fn exists(&self, entity: EntityId) -> bool {
        match self {
            World::Host(w) => w.exists(entity),
        }
    }

    /// Gets all of the entities that have the given `component`.
    #[doc(hidden)]
    pub fn get_all_untyped(&self, component: &dyn UntypedComponent) -> Vec<EntityId> {
        match self {
            World::Host(w) => w.get_all_untyped(component),
        }
    }

    /// Gets all of the entities within `radius` of `position`.
    pub fn in_area(&self, position: Vec3, radius: f32) -> Vec<EntityId> {
        match self {
            World::Host(w) => w.in_area(position, radius),
        }
    }

    /// Retrieves the component `component` for `entity` if it exists, or `None` if it doesn't.
    #[doc(hidden)]
    pub fn get_component_untyped(
        &self,
        entity: EntityId,
        component: &dyn UntypedComponent,
    ) -> Result<ComponentValue> {
        match self {
            World::Host(w) => w.get_component_untyped(entity, component),
        }
    }

    /// Retrieves the components `components` for `entity`. Will return an empty `Entity` if no components are found.
    pub fn get_components(&self, entity: EntityId, components: &[&dyn UntypedComponent]) -> Entity {
        match self {
            World::Host(w) => w.get_components(entity, components),
        }
    }

    /// Retrieves all guest-visible components for `entity`. Will return an empty `Entity` if no components are found.
    ///
    /// Note that this may not be all of the components on the entity, as some components are not visible to the guest.
    pub fn get_all_components(&self, entity: EntityId) -> Entity {
        match self {
            World::Host(w) => w.get_all_components(entity),
        }
    }

    /// Adds the component `component` for `entity` with `value`. Will replace an existing component if present.
    #[doc(hidden)]
    pub fn add_component_untyped(
        &mut self,
        entity: EntityId,
        component: &dyn UntypedComponent,
        value: ComponentValue,
    ) -> Result<()> {
        match self {
            World::Host(w) => w.add_component_untyped(entity, component, value),
        }
    }

    /// Adds the components `components` for `entity` with `value`. Will replace any existing components specified in `components`.
    pub fn add_components(&mut self, entity: EntityId, components: Entity) -> Result<()> {
        match self {
            World::Host(w) => w.add_components(entity, components),
        }
    }

    /// Sets the component `component` for `entity` with `value`.
    #[doc(hidden)]
    pub fn set_component_untyped(
        &mut self,
        entity: EntityId,
        component: &dyn UntypedComponent,
        value: ComponentValue,
    ) -> Result<()> {
        match self {
            World::Host(w) => w.set_component_untyped(entity, component, value),
        }
    }

    /// Sets the components `components` for `entity` with `value`.
    pub fn set_components(&mut self, entity: EntityId, components: Entity) {
        match self {
            World::Host(w) => w.set_components(entity, components),
        }
    }

    /// Checks if the `entity` has a `component`.
    #[doc(hidden)]
    pub fn has_component_untyped(
        &self,
        entity: EntityId,
        component: &dyn UntypedComponent,
    ) -> bool {
        match self {
            World::Host(w) => w.has_component_untyped(entity, component),
        }
    }

    /// Checks if the `entity` has `components`.
    pub fn has_components(&self, entity: EntityId, components: &[&dyn UntypedComponent]) -> bool {
        match self {
            World::Host(w) => w.has_components(entity, components),
        }
    }

    /// Removes the `component` from `entity`.
    ///
    /// Does nothing if the component does not exist.
    #[doc(hidden)]
    pub fn remove_component_untyped(&mut self, entity: EntityId, component: &dyn UntypedComponent) {
        match self {
            World::Host(w) => w.remove_component_untyped(entity, component),
        }
    }
    /// Removes the `components` from `entity`.
    ///
    /// Does nothing if the component does not exist.
    pub fn remove_components(&mut self, entity: EntityId, components: &[&dyn UntypedComponent]) {
        match self {
            World::Host(w) => w.remove_components(entity, components),
        }
    }

    /// Gets the resource entity. The components of this entity contain global state for this ECS world.
    ///
    /// Components with the `Resource` attribute can be found here.
    pub fn resources(&self) -> EntityId {
        match self {
            World::Host(w) => w.resources(),
        }
    }

    /// Gets the synchronized resource entity. The components of this entity contain global state that should be networked, but not persisted.
    pub fn synchronized_resources(&self) -> EntityId {
        match self {
            World::Host(w) => w.synchronized_resources(),
        }
    }

    /// Gets the persisted resource entity. The components of this entity contain global state that should be networked and persisted.
    pub fn persisted_resources(&self) -> EntityId {
        match self {
            World::Host(w) => w.persisted_resources(),
        }
    }
}

impl World {
    /// Gets all of the entities that have the given `component`.
    pub fn get_all<T: SupportedValue>(&self, component: Component<T>) -> Vec<EntityId> {
        self.get_all_untyped(&component)
    }

    /// Retrieves the component `component` for `entity` if it exists, or `None` if it doesn't.
    pub fn get_component<T: SupportedValue>(
        &self,
        entity: EntityId,
        component: Component<T>,
    ) -> Result<T> {
        self.get_component_untyped(entity, &component)
            .map(|x| T::from_value(x).unwrap())
    }
    /// Retrieves the component `component` for `entity` if it exists, or `None` if it doesn't.
    pub fn get<T: SupportedValue>(&self, entity: EntityId, component: Component<T>) -> Result<T> {
        self.get_component(entity, component)
    }
    #[doc(hidden)]
    pub fn get_cloned<T: SupportedValue>(
        &self,
        entity: EntityId,
        component: Component<T>,
    ) -> Result<T> {
        self.get_component(entity, component)
    }

    /// Retrieves the `resource` if it exists and panics if it doesn't.
    pub fn resource<T: SupportedValue>(&self, component: Component<T>) -> T {
        self.get_component(self.resources(), component).unwrap()
    }

    /// Adds the component `component` for `entity` with `value`. Will replace an existing component if present.
    pub fn add_component<T: SupportedValue>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
        value: T,
    ) -> Result<()> {
        self.add_component_untyped(entity, &component, value.into_value())
    }

    /// Sets the component `component` for `entity` with `value`.
    pub fn set_component<T: SupportedValue>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
        value: T,
    ) -> Result<()> {
        self.set_component_untyped(entity, &component, value.into_value())
    }
    /// Sets the component `component` for `entity` with `value`.
    pub fn set<T: SupportedValue>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
        value: T,
    ) -> Result<()> {
        self.set_component(entity, component, value)
    }

    /// Checks if the `entity` has a `component`.
    pub fn has_component<T: SupportedValue>(
        &self,
        entity: EntityId,
        component: Component<T>,
    ) -> bool {
        self.has_component_untyped(entity, &component)
    }

    /// Removes the `component` from `entity`.
    ///
    /// Does nothing if the component does not exist.
    pub fn remove_component<T: SupportedValue>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
    ) {
        self.remove_component_untyped(entity, &component)
    }

    /// Waits until `id` has the `component`. If the entity was deleted the method returns None.
    pub async fn wait_for_component<T: SupportedValue>(
        &self,
        entity: EntityId,
        component: Component<T>,
    ) -> Result<T> {
        block_until(move || !self.exists(entity) || self.has_component(entity, component)).await;
        self.get_component(entity, component)
    }

    /// Despawns `entity` and all of its children.
    pub fn despawn_recursive(&mut self, entity: EntityId) {
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
    pub fn mutate_component<T: SupportedValue + Clone + PartialEq>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
        mutator: impl FnOnce(&mut T),
    ) -> Result<T> {
        let mut value: T = self.get_component(entity, component)?;
        let orig_value = value.clone();
        mutator(&mut value);
        if value != orig_value {
            self.set_component(entity, component, value.clone());
        }
        Ok(value)
    }

    /// Mutates the component `component` for `entity` using the passed in `mutator`, or sets it
    /// to `default` if it doesn't exist, and returns its value.
    ///
    /// This will not set the component if the value is the same, which will prevent change events from
    /// being unnecessarily fired.
    pub fn mutate_component_with_default<T: SupportedValue + Clone + PartialEq>(
        &mut self,
        entity: EntityId,
        component: Component<T>,
        default: T,
        mutator: impl FnOnce(&mut T),
    ) -> T {
        let value = self.mutate_component(entity, component, mutator);
        if let Ok(value) = value {
            value
        } else {
            self.add_component(entity, component, default.clone());
            default
        }
    }

    /// Adds `child` as a child to `entity`.
    pub fn add_child(&mut self, entity: EntityId, child: EntityId) {
        if self.has_component(entity, children()) {
            self.mutate_component(entity, children(), |children| children.push(child));
        } else {
            self.add_component(entity, children(), vec![child]);
        }
        self.add_component(child, parent(), entity);
    }

    /// Removes `child` as a child to `entity`.
    pub fn remove_child(&mut self, entity: EntityId, child: EntityId) {
        if self.has_component(entity, children()) {
            self.mutate_component(entity, children(), |children| {
                children.retain(|x| *x != child)
            });
        }
        self.remove_component(child, parent());
    }
}
