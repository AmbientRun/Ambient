use crate::{
    global::{EntityId, Vec3},
    internal::{
        component::{Component, Entity, SupportedValue, UntypedComponent},
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
    prelude::block_until,
};

pub use wit::entity::{AnimationAction, AnimationController};

/// Spawns an entity containing the `components`.
///
/// Returns `spawned_entity_uid`.
pub fn spawn(components: &Entity) -> EntityId {
    // the function is too general to be passed in directly
    #[allow(clippy::redundant_closure)]
    components
        .call_with(|data| wit::entity::spawn(data))
        .from_bindgen()
}

/// Waits until `id` has the `component`. Note that this may never resolve if the entity
/// does not complete spawning, or the id in question refers to an entity that does
/// not exist.
pub async fn wait_for_component<T: SupportedValue>(entity: EntityId, component: Component<T>) {
    block_until(move || wit::component::has_component(entity.into_bindgen(), component.index()))
        .await;
}

/// Despawns `entity` from the world. `entity` will not work with any other functions afterwards.
///
/// Returns the data of the despawned entity, if it existed.
pub fn despawn(entity: EntityId) -> Option<Entity> {
    wit::entity::despawn(entity.into_bindgen()).from_bindgen()
}
/// Set the animation (controller) for `entity`.
pub fn set_animation_controller(entity: EntityId, controller: AnimationController) {
    wit::entity::set_animation_controller(entity.into_bindgen(), controller)
}

/// Checks if the `entity` exists.
pub fn exists(entity: EntityId) -> bool {
    wit::entity::exists(entity.into_bindgen())
}

/// Gets all of the entities that have the given `component`.
pub fn get_all<T>(component: Component<T>) -> Vec<EntityId> {
    wit::entity::get_all(component.index()).from_bindgen()
}

/// Gets all of the entities within `radius` of `position`.
pub fn in_area(position: Vec3, radius: f32) -> Vec<EntityId> {
    wit::entity::in_area(position.into_bindgen(), radius).from_bindgen()
}

/// Retrieves the component `component` for `entity` if it exists, or `None` if it doesn't.
pub fn get_component<T: SupportedValue>(entity: EntityId, component: Component<T>) -> Option<T> {
    T::from_result(wit::component::get_component(
        entity.into_bindgen(),
        component.index(),
    )?)
}

/// Adds the component `component` for `entity` with `value`. Will replace an existing component if present.
pub fn add_component<T: SupportedValue>(entity: EntityId, component: Component<T>, value: T) {
    wit::component::add_component(
        entity.into_bindgen(),
        component.index(),
        &value.into_result(),
    )
}

/// Adds the components `components` for `entity` with `value`. Will replace any existing components specified in `components`.
pub fn add_components(entity: EntityId, components: Entity) {
    components.call_with(|data| wit::component::add_components(entity.into_bindgen(), data))
}

/// Sets the component `component` for `entity` with `value`.
pub fn set_component<T: SupportedValue>(entity: EntityId, component: Component<T>, value: T) {
    wit::component::set_component(
        entity.into_bindgen(),
        component.index(),
        &value.into_result(),
    )
}

/// Sets the components `components` for `entity` with `value`.
pub fn set_components(entity: EntityId, components: Entity) {
    components.call_with(|data| wit::component::set_components(entity.into_bindgen(), data))
}

/// Checks if the `entity` has a `component`.
pub fn has_component<T: SupportedValue>(entity: EntityId, component: Component<T>) -> bool {
    wit::component::has_component(entity.into_bindgen(), component.index())
}

/// Checks if the `entity` has `components`.
pub fn has_components(entity: EntityId, components: &[&dyn UntypedComponent]) -> bool {
    let components: Vec<_> = components.iter().map(|c| c.index()).collect();
    wit::component::has_components(entity.into_bindgen(), &components)
}

/// Adds the `component` with `value` to `entity` if `entity` does not already have that component.
pub fn add_component_if_required<T: SupportedValue + SupportedValue>(
    entity: EntityId,
    component: Component<T>,
    value: T,
) {
    if !has_component(entity, component) {
        add_component(entity, component, value)
    }
}

/// Removes the `component` from `entity`.
///
/// Does nothing if the component does not exist.
pub fn remove_component<T>(entity: EntityId, component: Component<T>) {
    wit::component::remove_component(entity.into_bindgen(), component.index())
}

/// Removes the `components` from `entity`.
///
/// Does nothing if the component does not exist.
pub fn remove_components(entity: EntityId, components: &[&dyn UntypedComponent]) {
    let components: Vec<_> = components.iter().map(|c| c.index()).collect();
    wit::component::remove_components(entity.into_bindgen(), &components)
}

/// Mutates the component `name` for `entity` using the passed in `mutator`, and returns its value.
///
/// This will not set the component if the value is the same, which will prevent change events from
/// being unnecessarily fired.
pub fn mutate_component<T: SupportedValue + SupportedValue + Clone + PartialEq>(
    entity: EntityId,
    component: Component<T>,
    mutator: impl FnOnce(&mut T),
) -> Option<T> {
    let mut value: T = get_component(entity, component)?;
    let orig_value = value.clone();
    mutator(&mut value);
    if value != orig_value {
        set_component(entity, component, value.clone());
    }
    Some(value)
}

/// Mutates the component `name` for `entity` using the passed in `mutator`, or sets it
/// to `default` if it doesn't exist, and returns its value.
///
/// This will not set the component if the value is the same, which will prevent change events from
/// being unnecessarily fired.
pub fn mutate_component_with_default<T: SupportedValue + SupportedValue + Clone + PartialEq>(
    entity: EntityId,
    component: Component<T>,
    default: T,
    mutator: impl FnOnce(&mut T),
) -> T {
    let value = mutate_component(entity, component, mutator);
    if let Some(value) = value {
        value
    } else {
        set_component(entity, component, default.clone());
        default
    }
}

/// Gets the resource entity. The components of this entity contain global state for this ECS world.
///
/// Components with the `Resource` attribute can be found here.
pub fn resources() -> EntityId {
    EntityId::resources()
}

/// Gets the synchronized resource entity. The components of this entity contain global state that should be networked, but not persisted.
pub fn synchronized_resources() -> EntityId {
    wit::entity::synchronized_resources().from_bindgen()
}

/// Gets the persisted resource entity. The components of this entity contain global state that should be networked and persisted.
pub fn persisted_resources() -> EntityId {
    wit::entity::persisted_resources().from_bindgen()
}
