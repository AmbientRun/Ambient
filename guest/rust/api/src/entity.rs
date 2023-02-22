use crate::{
    components, event,
    global::{until_this, EntityId, Vec3},
    internal::{
        component::{
            traits::AsParam, Component, Entity, SupportedComponentTypeGet,
            SupportedComponentTypeSet, UntypedComponent,
        },
        conversion::{FromBindgen, IntoBindgen},
        host,
    },
};

pub use crate::internal::host::{AnimationAction, AnimationController};

/// Spawns an entity containing the `components`.
///
/// This is an asynchronous operation; use [wait_for_spawn] to get notified when
/// the entity is spawned.
///
/// Returns `spawned_entity_uid`.
pub fn spawn(components: &Entity) -> EntityId {
    components
        .call_with(|data| host::entity_spawn(data))
        .from_bindgen()
}

/// Waits until `id` has spawned. Note that this may never resolve if the entity
/// does not complete spawning, or the id in question refers to an entity that does
/// not exist.
// TODO(philpax): revisit once we think about the spawning situation some more
pub async fn wait_for_spawn(id: EntityId) {
    until_this(event::ENTITY_SPAWN, move |ed| {
        ed.get(components::core::ecs::id()).unwrap() == id
    })
    .await;
}

/// Despawns `entity` from the world. `entity` will not work with any other functions afterwards.
///
/// Returns whether or not the entity was removed.
pub fn despawn(entity: EntityId) -> bool {
    host::entity_despawn(entity.into_bindgen())
}
/// Set the animation (controller) for `entity`.
pub fn set_animation_controller(entity: EntityId, controller: AnimationController) {
    host::entity_set_animation_controller(entity.into_bindgen(), controller)
}

/// Checks if the `entity` exists.
pub fn exists(entity: EntityId) -> bool {
    host::entity_exists(entity.into_bindgen())
}

/// Gets all of the entities that have the given `component`.
pub fn get_all<T>(component: Component<T>) -> Vec<EntityId> {
    host::entity_get_all(component.index()).from_bindgen()
}

/// Gets all of the entities within `radius` of `position`.
pub fn in_area(position: Vec3, radius: f32) -> Vec<EntityId> {
    host::entity_in_area(position.into_bindgen(), radius).from_bindgen()
}

/// Retrieves the component `component` for `entity` if it exists, or `None` if it doesn't.
pub fn get_component<T: SupportedComponentTypeGet>(
    entity: EntityId,
    component: Component<T>,
) -> Option<T> {
    T::from_result(host::entity_get_component(
        entity.into_bindgen(),
        component.index(),
    )?)
}

/// Adds the component `component` for `entity` with `value`. Will replace an existing component if present.
pub fn add_component<T: SupportedComponentTypeSet>(
    entity: EntityId,
    component: Component<T>,
    value: T,
) {
    let owned = value.into_owned_param();
    host::entity_add_component(entity.into_bindgen(), component.index(), owned.as_param())
}

/// Adds the components `components` for `entity` with `value`. Will replace any existing components specified in `components`.
pub fn add_components(entity: EntityId, components: Entity) {
    components.call_with(|data| host::entity_add_components(entity.into_bindgen(), data))
}

/// Sets the component `component` for `entity` with `value`.
pub fn set_component<T: SupportedComponentTypeSet>(
    entity: EntityId,
    component: Component<T>,
    value: T,
) {
    let owned = value.into_owned_param();
    host::entity_set_component(entity.into_bindgen(), component.index(), owned.as_param())
}

/// Sets the components `components` for `entity` with `value`.
pub fn set_components(entity: EntityId, components: Entity) {
    components.call_with(|data| host::entity_set_components(entity.into_bindgen(), data))
}

/// Checks if the `entity` has a `component`.
pub fn has_component<T: SupportedComponentTypeGet>(
    entity: EntityId,
    component: Component<T>,
) -> bool {
    host::entity_has_component(entity.into_bindgen(), component.index())
}

/// Checks if the `entity` has `components`.
pub fn has_components(entity: EntityId, components: &[&dyn UntypedComponent]) -> bool {
    let components: Vec<_> = components.iter().map(|c| c.index()).collect();
    host::entity_has_components(entity.into_bindgen(), &components)
}

/// Adds the `component` with `value` to `entity` if `entity` does not already have that component.
pub fn add_component_if_required<T: SupportedComponentTypeGet + SupportedComponentTypeSet>(
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
    host::entity_remove_component(entity.into_bindgen(), component.index())
}

/// Removes the `components` from `entity`.
///
/// Does nothing if the component does not exist.
pub fn remove_components(entity: EntityId, components: &[&dyn UntypedComponent]) {
    let components: Vec<_> = components.iter().map(|c| c.index()).collect();
    host::entity_remove_components(entity.into_bindgen(), &components)
}

/// Mutates the component `name` for `entity` using the passed in `mutator`, and returns its value.
///
/// This will not set the component if the value is the same, which will prevent change events from
/// being unnecessarily fired.
pub fn mutate_component<
    T: SupportedComponentTypeGet + SupportedComponentTypeSet + Clone + PartialEq,
>(
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
pub fn mutate_component_with_default<
    T: SupportedComponentTypeGet + SupportedComponentTypeSet + Clone + PartialEq,
>(
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

/// Gets the resource entity which contains global state in its components.
///
/// Components with the `Resource` attribute can be found here.
pub fn resources() -> EntityId {
    host::entity_resources().from_bindgen()
}
