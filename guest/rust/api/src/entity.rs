use crate::{
    components, event,
    global::{until_this, EntityId, EntityUid, Mat4, ObjectRef, Quat, Vec3},
    internal::{
        component::{
            traits::AsParam, Component, Components, SupportedComponentTypeGet,
            SupportedComponentTypeSet, UntypedComponent,
        },
        conversion::{FromBindgen, IntoBindgen},
        host,
    },
};

pub use crate::internal::host::{AnimationAction, AnimationController};

/// Spawns an entity containing the `components`. If `persistent` is set, this entity will not be
/// removed when this module is unloaded.
///
/// This is an asynchronous operation; use [wait_for_spawn] to get notified when
/// the entity is spawned.
///
/// Returns `spawned_entity_uid`.
pub fn spawn(components: &Components, persistent: bool) -> EntityUid {
    components
        .call_with(|data| host::entity_spawn(data, persistent))
        .from_bindgen()
}

/// Waits until `uid` has fully spawned. Note that this may never resolve if the entity
/// does not complete spawning, or the UID in question refers to an entity that does
/// not exist.
// TODO(philpax): revisit once we think about the spawning situation some more
pub async fn wait_for_spawn(uid: &EntityUid) -> EntityId {
    let uid = uid.clone();
    let event = until_this(event::ENTITY_SPAWN, move |ed| {
        ed.get(components::core::ecs::uid()).unwrap() == uid
    })
    .await;
    event.get(components::core::ecs::id()).unwrap()
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
/// Gets the position of the `entity` if it exists, or `None` if it does not.
pub fn get_position(entity: EntityId) -> Option<Vec3> {
    get_component(entity, components::core::transform::translation())
}
/// Sets the position of `entity` to `position`.
pub fn set_position(entity: EntityId, position: Vec3) {
    let (rotation, scale) = (
        get_rotation(entity).unwrap_or_default(),
        get_scale(entity).unwrap_or(Vec3::ONE),
    );

    host::entity_set_transform(
        entity.into_bindgen(),
        Mat4::from_scale_rotation_translation(scale, rotation, position).into_bindgen(),
        false,
    );
}
/// Gets the rotation of the `entity` if it exists, or `None` if it does not.
pub fn get_rotation(entity: EntityId) -> Option<Quat> {
    get_component(entity, components::core::transform::rotation())
}
/// Sets the rotation of `entity` to `rotation`.
pub fn set_rotation(entity: EntityId, rotation: Quat) {
    let (translation, scale) = (
        get_position(entity).unwrap_or_default(),
        get_scale(entity).unwrap_or(Vec3::ONE),
    );

    host::entity_set_transform(
        entity.into_bindgen(),
        Mat4::from_scale_rotation_translation(scale, rotation, translation).into_bindgen(),
        false,
    );
}
/// Gets the scale of the `entity` if it exists, or `None` if it does not.
pub fn get_scale(entity: EntityId) -> Option<Vec3> {
    get_component(entity, components::core::transform::scale())
}
/// Sets the scale of `entity` to `scale`.
pub fn set_scale(entity: EntityId, scale: Vec3) {
    let (rotation, translation) = (
        get_rotation(entity).unwrap_or_default(),
        get_position(entity).unwrap_or_default(),
    );

    host::entity_set_transform(
        entity.into_bindgen(),
        Mat4::from_scale_rotation_translation(scale, rotation, translation).into_bindgen(),
        false,
    );
}
/// Sets the `transform` matrix of the `entity` (i.e. position, rotation and scale at the same time).
pub fn set_transform(entity: EntityId, transform: glam::Affine3A) {
    let transform: Mat4 = transform.into();
    host::entity_set_transform(entity.into_bindgen(), transform.into_bindgen(), false);
}
/// Applies `transform` to the `entity` (i.e. moving / rotating / scaling where it currently is).
pub fn transform_by(entity: EntityId, transform: glam::Affine3A) {
    let transform: Mat4 = transform.into();
    host::entity_set_transform(entity.into_bindgen(), transform.into_bindgen(), true);
}

/// Gets the linear velocity of `entity` if it exists, or `None` if it does not.
pub fn get_linear_velocity(entity: EntityId) -> Option<Vec3> {
    host::entity_get_linear_velocity(entity.into_bindgen()).from_bindgen()
}

/// Checks if the `entity` exists.
pub fn exists(entity: EntityId) -> bool {
    host::entity_exists(entity.into_bindgen())
}

/// Gets all of the entities that have the given `component`.
pub fn query<T>(component: Component<T>) -> Vec<EntityId> {
    host::entity_query(component.index()).from_bindgen()
}

/// Get the [EntityId] for the specified [EntityUid], if available.
pub fn lookup_uid(uid: &EntityUid) -> Option<EntityId> {
    host::entity_lookup_uid(uid.into_bindgen()).from_bindgen()
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

/// Sets the component `component` for `entity` with `value`.
pub fn set_component<T: SupportedComponentTypeSet>(
    entity: EntityId,
    component: Component<T>,
    value: T,
) {
    let owned = value.into_owned_param();
    host::entity_set_component(entity.into_bindgen(), component.index(), owned.as_param())
}

/// Sets the component `components` for `entity` with `value`.
pub fn set_components(entity: EntityId, components: Components) {
    components.call_with(|data| host::entity_set_components(entity.into_bindgen(), data))
}

/// Checks if the `entity` has a `component`.
pub fn has_component<T: SupportedComponentTypeGet>(
    entity: EntityId,
    component: Component<T>,
) -> bool {
    host::entity_has_component(entity.into_bindgen(), component.index())
}

/// Adds the `component` with `value` to `entity` if `entity` does not already have that component.
pub fn add_component_if_required<T: SupportedComponentTypeGet + SupportedComponentTypeSet>(
    entity: EntityId,
    component: Component<T>,
    value: T,
) {
    if !has_component(entity, component) {
        set_component(entity, component, value)
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

/// Creates a [Components] that can be used as a base for a game object.
pub fn game_object_base() -> Components {
    Components::new()
        .with(components::core::transform::translation(), Vec3::ZERO)
        .with(components::core::transform::rotation(), Quat::IDENTITY)
        .with(components::core::transform::scale(), Vec3::ONE)
}
