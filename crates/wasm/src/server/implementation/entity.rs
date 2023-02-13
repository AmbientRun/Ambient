use glam::{Mat4, Quat, Vec3};
use kiwi_animation::{animation_controller, AnimationController};
use kiwi_ecs::{EntityData, EntityId, World};

use kiwi_physics::helpers as eph;

pub fn spawn(world: &mut World, data: EntityData) -> EntityId {
    data.spawn(world)
}

pub fn despawn(world: &mut World, entity: EntityId) -> Option<EntityId> {
    world.despawn(entity).and_then(|ed| Some(entity))
}

pub fn set_transform(
    world: &mut World,
    entity: EntityId,
    transform: Mat4,
    relative: bool,
) -> anyhow::Result<()> {
    Ok(eph::transform_entity(world, entity, transform, relative)?)
}

pub fn get_linear_velocity(world: &mut World, entity: EntityId) -> anyhow::Result<Vec3> {
    Ok(eph::get_linear_velocity(world, entity)?)
}

pub fn set_animation_controller(
    world: &mut World,
    entity: EntityId,
    controller: AnimationController,
) -> anyhow::Result<()> {
    Ok(world.add_component(entity, animation_controller(), controller)?)
}
