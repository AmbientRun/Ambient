use std::marker::PhantomData;

pub use crate::internal::component::{
    query::{
        change_query, despawn_query, query, spawn_query, ChangeQuery, EventQuery, GeneralQuery,
        GeneralQueryBuilder, QueryEvent, UntrackedChangeQuery,
    },
    Component, ComponentOptionValue, ComponentValue, ComponentVecValue, ComponentsTuple, Entity,
    EnumComponent, SupportedValue, UntypedComponent, __internal_get_component,
};

use ambient_shared_types::ComponentIndex;

#[doc(hidden)]
pub use crate::internal::wit::component::Value as WitComponentValue;
use crate::prelude::EntityId;

/// Concepts are defined in the package manifest, and are used to define a collection of components that correspond to some concept in the game world.
///
/// For example, a `Camera` concept might describe a camera in the game world, and have a `near` and `projection` component.
pub trait Concept {
    /// Creates an entity with the components defined by this concept.
    fn make(self) -> Entity;

    /// Spawns this concept into the world. If you want to modify state before spawning, use `make` instead.
    fn spawn(self) -> EntityId
    where
        Self: Sized,
    {
        self.make().spawn()
    }

    /// If the entity with `id` exists and has the components defined by this concept, returns this concept with all of the values of the components in the entity.
    ///
    /// # Examples
    /// ```
    /// if let Some(camera) = Camera::get_spawned(id) {
    ///    println!("{}", camera.near);
    /// }
    /// ```
    fn get_spawned(id: EntityId) -> Option<Self>
    where
        Self: Sized;
    /// If the `entity` has the components defined by this concept, returns this concept with all of the values of the components in the entity.
    ///
    /// # Examples
    /// ```
    /// if let Some(camera) = Camera::get_unspawned(ent) {
    ///    println!("{}", camera.near);
    /// }
    /// ```
    fn get_unspawned(entity: &Entity) -> Option<Self>
    where
        Self: Sized;

    /// Returns true if `id` exists and contains the components defined by this concept.
    ///
    /// # Examples
    /// ```
    /// if Camera::contained_by_spawned(id) {
    ///    // ...
    /// }
    /// ```
    fn contained_by_spawned(id: EntityId) -> bool;
    /// Returns true if contains the components defined by this concept.
    ///
    /// # Examples
    /// ```
    /// if Camera::contained_by_unspawned(ent) {
    ///    // ...
    /// }
    /// ```
    fn contained_by_unspawned(entity: &Entity) -> bool;
}
impl<T: Concept + Sized> From<T> for Entity {
    fn from(concept: T) -> Self {
        concept.make()
    }
}
/// Provides a helper method to get an instance of this concept with all of the fields
/// filled in with suggested values.
///
/// This trait is only implemented if all fields in a concept have a suggested value.
pub trait ConceptSuggested: Concept {
    /// Returns this concept with all of its fields filled in with suggested values.
    ///
    /// The optional field, if present, will be defaulted/have all of its fields be `None`.
    fn suggested() -> Self;
}
/// Provides component tuples for this concept.
pub trait ConceptComponents: Concept {
    /// A tuple of the required components for this concept.
    type Required: ComponentsTuple;
    /// A tuple of the optional components for this concept.
    type Optional: ComponentsTuple;

    /// Returns a tuple of the required components for this concept.
    fn required() -> Self::Required;
    /// Returns a tuple of the optional components for this concept.
    fn optional() -> Self::Optional;
    /// Converts a tuple of data back to a concept.
    fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self;

    /// Creates a [`ConceptQuery`] that can be passed into queries.
    ///
    /// Note that this will only get the required components of the concept, and not the optional
    /// components!
    fn as_query() -> ConceptQuery<Self>
    where
        Self: Sized,
    {
        ConceptQuery(PhantomData)
    }
}

/// Helper that lets you pass in concepts where component tuples are expected.
///
/// Note that this will only get the required components of the concept, and not the optional
/// components!
// TODO: See if we can revise the APIs to remove this.
#[derive(Default, Debug)]
pub struct ConceptQuery<C: ConceptComponents>(PhantomData<C>);
impl<C: ConceptComponents> Copy for ConceptQuery<C> {}
impl<C: ConceptComponents> Clone for ConceptQuery<C> {
    fn clone(&self) -> Self {
        *self
    }
}
/// Helper blanket implementation that allows you to use concepts where component tuples are expected.
impl<C: ConceptComponents> ComponentsTuple for ConceptQuery<C> {
    type Data = C;

    fn as_indices(&self) -> Vec<ComponentIndex> {
        C::required().as_indices()
    }

    fn from_component_types(
        component_types: Vec<crate::internal::wit::component::Value>,
    ) -> Option<Self::Data> {
        Some(C::from_required_data(
            <C as ConceptComponents>::Required::from_component_types(component_types)?,
        ))
    }
}
