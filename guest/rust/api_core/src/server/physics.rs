use crate::{
    global::{EntityId, Vec3},
    internal::{
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
};
use glam::Mat4;

/// Applies a `force` (a [Vec3]) to the `entity` (an [EntityId]) specified.
///
/// `force` is a vector in world space, which means it has both direction and magnitude. To push objects upwards
/// (positive Z) with strength 3,000, you would supply a force of `vec3(0.0, 0.0, 3_000.0)` or
/// `Vec3::Z * 3_000.0` (either are equivalent.)
pub fn add_force(entity: EntityId, force: Vec3) {
    wit::server_physics::add_force(entity.into_bindgen(), force.into_bindgen())
}

/// Applies an `impulse` (a [Vec3]) to the `entity` (an [EntityId]) specified.
///
/// `impulse` is a vector in world space, which means it has both direction and magnitude. To push objects upwards
/// (positive Z) with strength 3,000, you would supply an impulse of `vec3(0.0, 0.0, 3_000.0)` or
/// `Vec3::Z * 3_000.0` (either are equivalent.)
pub fn add_impulse(entity: EntityId, impulse: Vec3) {
    wit::server_physics::add_force(entity.into_bindgen(), impulse.into_bindgen())
}

/// Whether or not to apply a falloff to the strength of [add_radial_impulse].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FalloffRadius {
    /// No falloff. The impulse will be of equal strength for all entities.
    #[default]
    None,
    /// Applies a falloff to the strength of the impulse, so that it drops out the further out the object is,
    /// until it reaches a strength of 0 at `falloff_radius`.
    FalloffToZeroAt(f32),
}
impl From<FalloffRadius> for Option<f32> {
    fn from(radius: FalloffRadius) -> Option<f32> {
        match radius {
            FalloffRadius::None => None,
            FalloffRadius::FalloffToZeroAt(r) => Some(r),
        }
    }
}

/// Applies an `impulse` (a [f32]) outwards to all entitities within `radius` of the `position`, with
/// an optional `falloff_radius`.
pub fn add_radial_impulse(
    position: Vec3,
    impulse: f32,
    radius: f32,
    falloff_radius: FalloffRadius,
) {
    wit::server_physics::add_radial_impulse(
        position.into_bindgen(),
        impulse,
        radius,
        falloff_radius.into(),
    )
}

/// Applies a `force` (a [Vec3]) at a given `position` (a [Vec3]) to the `entity` (an [EntityId]) specified.
///
/// `force` is a vector in world space, which means it has both direction and magnitude. To push objects upwards
/// (positive Z) with strength 3,000, you would supply a force of `vec3(0.0, 0.0, 3_000.0)` or
/// `Vec3::Z * 3_000.0` (either are equivalent.)
///
/// `position` is a position in world space, it typically should fall on the surface or interior of an object for
/// realistic results.
pub fn add_force_at_position(entity: EntityId, force: Vec3, position: Vec3) {
    wit::server_physics::add_force_at_position(
        entity.into_bindgen(),
        force.into_bindgen(),
        position.into_bindgen(),
    )
}

/// Applies an `impulse` (a [Vec3]) at given `position` (a [Vec3]) to the `entity` (an [EntityId]) specified.
///
/// `impulse` is a vector in world space, which means it has both direction and magnitude. To push objects upwards
/// (positive Z) with strength 3,000, you would supply an impulse of `vec3(0.0, 0.0, 3_000.0)` or
/// `Vec3::Z * 3_000.0` (either are equivalent.)
///
/// `position` is a position in world space, it typically should fall on the surface or interior of an object for
/// realistic results.
pub fn add_impulse_at_position(entity: EntityId, impulse: Vec3, position: Vec3) {
    wit::server_physics::add_impulse_at_position(
        entity.into_bindgen(),
        impulse.into_bindgen(),
        position.into_bindgen(),
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

/// Creates a revolute joint. entity0 or entity1 can either be `EntityId::null()` to bind this to the world frame.
pub fn create_revolute_joint(
    entity0: EntityId,
    transform0: Mat4,
    entity1: EntityId,
    transform1: Mat4,
) {
    wit::server_physics::create_revolute_joint(
        entity0.into_bindgen(),
        transform0.into_bindgen(),
        entity1.into_bindgen(),
        transform1.into_bindgen(),
    )
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

/// Move an entity with a character collider on it, by sweeping the collider.
/// This will ensure that it collides with any objects in its path.
///
/// A character collider can be added to an entity using the `character_controller` concept.
///
/// You can also update the entity's [translation](crate::core::transform::components::translation) component,
/// but this will teleport it to that location.
///
/// Arguments:
///  - `displacement`: The displacement to move the character by.
///  - `min_dist`: The minimum travelled distance to consider. If travelled distance is smaller, the character doesn't move. This is used to stop the recursive motion algorithm when remaining distance to travel is small.
///  - `elapsed_time`: The elapsed time since last call to this function.
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

/// Set character controller position
pub fn set_character_position(entity: EntityId, position: Vec3) {
    wit::server_physics::set_character_position(entity.into_bindgen(), position.into_bindgen());
}

/// Set character controller foot position
pub fn set_character_foot_position(entity: EntityId, position: Vec3) {
    wit::server_physics::set_character_foot_position(
        entity.into_bindgen(),
        position.into_bindgen(),
    );
}
