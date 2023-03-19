use crate::{
    global::{EntityId, Vec3},
    internal::{
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
};

pub use crate::internal::wit::server_physics::ForceMode;

/// Applies a `force` (a [Vec3]) with a given `mode` (a [ForceMode]) to the `entity` (an [EntityId]) specified.
///
/// `force` is a vector in world space, which means it has both direction and magnitude. To push objects upwards
/// (positive Z) with strength 3,000, you would supply a force of `vec3(0.0, 0.0, 3_000.0)` or
/// `Vec3::Z * 3_000.0` (either are equivalent.)
///
/// `mode` is an enum, which determines how a given force should be applied. See the associated documentation
/// for more information. Defaults to `ForceMode::Force`.
pub fn apply_force(entity: EntityId, force: Vec3, mode: Option<ForceMode>) {
    wit::server_physics::apply_force(entity.into_bindgen(), force.into_bindgen(), mode)
}

/// Applies a `force` (a [Vec3]) with a given `mode` (a [ForceMode]) and `position` (a [Vec3]) to the `entity` (an [EntityId]) specified.
///
/// `force` is a vector in world space, which means it has both direction and magnitude. To push objects upwards
/// (positive Z) with strength 3,000, you would supply a force of `vec3(0.0, 0.0, 3_000.0)` or
/// `Vec3::Z * 3_000.0` (either are equivalent.)
///
/// `position` is a position in world space, it typically should fall on the surface or interior of an object for
/// realistic results.
///
/// `mode` is an enum, which determines how a given force should be applied. See the associated documentation
/// for more information. Defaults to `ForceMode::Force`.
pub fn apply_force_at_position(
    entity: EntityId,
    force: Vec3,
    position: Vec3,
    mode: Option<ForceMode>,
) {
    wit::server_physics::apply_force_at_position(
        entity.into_bindgen(),
        force.into_bindgen(),
        position.into_bindgen(),
        mode,
    )
}

/// Gets the velocity (a [Vec3]) at a given `position` (a [Vec3]) of an `entity` (an [EntityId]) taking its
/// angular velocity into account.
///
/// `position` is a position in world space, it typically should fall on the surface or interior of an object.
pub fn get_velocity_at_position(entity: EntityId, position: Vec3) -> Vec3 {
    wit::server_physics::get_velocity_at_position(entity.into_bindgen(), position.into_bindgen())
        .from_bindgen()
}

/// Applies a `force` (a [f32]) outwards to all entitities within `radius` of the `position`, with
/// an optional `falloff_radius`.
///
/// If `fallout_radius` is specified (e.g. `Some(5_000)`), the strength of the explosion
/// will drop out the further out the object is, until it reaches a strength of 0 at `fallout_radius`.
///
/// Otherwise, the force will be of equal strength for all entities.
// TODO: consider making `fallout_radius` an enum, so the behaviour is explicit
pub fn explode_bomb(position: Vec3, force: f32, radius: f32, falloff_radius: Option<f32>) {
    wit::server_physics::explode_bomb(position.into_bindgen(), force, radius, falloff_radius)
}

/// Sets the gravity of the entire world to `gravity`. The default `gravity` is `vec3(0.0, 0.0, -9.82)`.
///
/// This can be used to simulate a different gravity from Earth's, or to create unconventional gameplay.
pub fn set_gravity(gravity: Vec3) {
    wit::server_physics::set_gravity(gravity.into_bindgen())
}

/// Unfreezes a frozen `entity`, so that it can move around. Does nothing if the entity wasn't frozen.
pub fn unfreeze(entity: EntityId) {
    wit::server_physics::unfreeze(entity.into_bindgen())
}

/// Freezes an `entity`, so that it cannot move around. Does nothing if the entity was already frozen.
pub fn freeze(entity: EntityId) {
    wit::server_physics::freeze(entity.into_bindgen())
}

/// Starts a motor on `entity` with `velocity`. Does nothing if the motor has already been started.
pub fn start_motor(entity: EntityId, velocity: f32) {
    wit::server_physics::start_motor(entity.into_bindgen(), velocity)
}

/// Stops a motor on `entity`. Does nothing if the motor is not running.
pub fn stop_motor(entity: EntityId) {
    wit::server_physics::stop_motor(entity.into_bindgen())
}

/// Where a [raycast] hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaycastHit {
    /// The position of the hit.
    pub position: Vec3,
    /// The distance from the origin to the hit.
    pub distance: f32,
    /// The entity that was hit.
    pub entity: EntityId,
}
/// Casts a ray from `origin` in `direction`, and returns the [RaycastHit]s along the way.
///
/// `direction` must be normalized.
pub fn raycast(origin: Vec3, direction: Vec3) -> Vec<RaycastHit> {
    wit::server_physics::raycast(origin.into_bindgen(), direction.into_bindgen())
        .into_iter()
        .map(|(entity, distance)| raycast_result_to_hit(origin, direction, entity, distance))
        .collect()
}
/// Casts a ray from `origin` in `direction`, and returns the first [RaycastHit] if it hits.
///
/// `direction` must be normalized.
pub fn raycast_first(origin: Vec3, direction: Vec3) -> Option<RaycastHit> {
    wit::server_physics::raycast_first(origin.into_bindgen(), direction.into_bindgen())
        .map(|(entity, distance)| raycast_result_to_hit(origin, direction, entity, distance))
}
fn raycast_result_to_hit(
    origin: Vec3,
    direction: Vec3,
    entity: wit::types::EntityId,
    distance: f32,
) -> RaycastHit {
    RaycastHit {
        position: origin + direction * distance,
        distance,
        entity: entity.from_bindgen(),
    }
}

/// Collision results when using [move_character].
pub struct CharacterCollision {
    /// Side
    pub side: bool,
    /// Up
    pub up: bool,
    /// Down
    pub down: bool,
}

/// Move an entity with a character collider on it, by sweeping the collider. This will ensure that it collides with any
/// objects in its path. You can also update the entity's [translation](crate::components::core::transform::translation) component, but this will teleport it to that location.
pub fn move_character(
    entity: EntityId,
    displacement: Vec3,
    min_dist: f32,
    elapsed_time: f32,
) -> CharacterCollision {
    let res = wit::server_physics::move_character(
        entity.into_bindgen(),
        displacement.into_bindgen(),
        min_dist,
        elapsed_time,
    );
    CharacterCollision {
        side: res.side,
        up: res.up,
        down: res.down,
    }
}
