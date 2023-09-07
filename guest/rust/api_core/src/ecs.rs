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
use crate::prelude::EntityId;

/// Concepts are defined in the package manifest, and are used to define a collection of components that correspond to some concept in the game world.
///
/// For example, a `Camera` concept might describe a camera in the game world, and have a `near` and `projection` component.
pub trait Concept {
    /// Creates an entity with the components defined by this concept.
    fn make(&self) -> Entity;

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
    fn get_unspawned(entity: Entity) -> Option<Self>
    where
        Self: Sized;

    /// Returns `Some(id)` if `id` exists and contains the components defined by this concept.
    ///
    /// # Examples
    /// ```
    /// if let Some(id) = Camera::contained_by_spawned(id) {
    ///    // ...
    /// }
    /// ```
    fn contained_by_spawned(id: EntityId) -> Option<EntityId>;
    /// Returns `Some(entity)` if contains the components defined by this concept.
    ///
    /// # Examples
    /// ```
    /// if let Some(ent) = Camera::contained_by_unspawned(ent) {
    ///    // ...
    /// }
    /// ```
    fn contained_by_unspawned(entity: Entity) -> Option<Entity>;
}
impl<T: Concept + Sized> From<T> for Entity {
    fn from(concept: T) -> Self {
        concept.make()
    }
}
