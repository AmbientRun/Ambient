use elements_animation::{animation_controller, AnimationController};
use elements_ecs::{EntityId, World};
use elements_physics::helpers as eph;
use glam::{Mat4, Vec3};

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
