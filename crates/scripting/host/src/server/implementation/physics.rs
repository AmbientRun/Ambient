use anyhow::Context;
use elements_ecs::{EntityId, World};
use elements_physics::{
    helpers::{self as eph, PhysicsObjectCollection},
    main_physics_scene,
};
use elements_std::shapes::Ray;
use glam::Vec3;
use physxx::PxRevoluteJointFlag;

pub fn apply_force(
    world: &mut World,
    entities: PhysicsObjectCollection,
    force: Vec3,
) -> anyhow::Result<()> {
    entities.apply_force(world, |_| force);
    Ok(())
}

pub fn explode_bomb(
    world: &mut World,
    position: Vec3,
    radius: f32,
    force: f32,
    falloff_radius: Option<f32>,
) -> anyhow::Result<()> {
    eph::PhysicsObjectCollection::from_radius(world, position, radius).apply_force_explosion(
        world,
        position,
        force,
        falloff_radius,
    );
    Ok(())
}

pub fn set_gravity(world: &mut World, gravity: Vec3) -> anyhow::Result<()> {
    world.resource(main_physics_scene()).set_gravity(gravity);
    Ok(())
}

pub fn unfreeze(world: &mut World, id: EntityId) -> anyhow::Result<()> {
    eph::convert_rigid_static_to_dynamic(world, id);
    Ok(())
}

pub fn freeze(world: &mut World, id: EntityId) -> anyhow::Result<()> {
    eph::convert_rigid_dynamic_to_static(world, id);
    Ok(())
}

pub fn start_motor(world: &mut World, id: EntityId, velocity: f32) -> anyhow::Result<()> {
    let joint = eph::get_entity_revolute_joint(world, id).context("Entity doesn't have a motor")?;
    joint.set_drive_velocity(velocity, true);
    joint.set_revolute_flag(PxRevoluteJointFlag::DRIVE_ENABLED, true);

    Ok(())
}

pub fn stop_motor(world: &mut World, id: EntityId) -> anyhow::Result<()> {
    let joint = eph::get_entity_revolute_joint(world, id).context("Entity doesn't have a motor")?;
    joint.set_revolute_flag(PxRevoluteJointFlag::DRIVE_ENABLED, false);

    Ok(())
}

pub fn raycast_first(
    world: &World,
    origin: Vec3,
    direction: Vec3,
) -> anyhow::Result<Option<(EntityId, f32)>> {
    Ok(elements_physics::intersection::raycast_first(
        world,
        Ray::new(origin, direction),
    ))
}

pub fn raycast(
    world: &World,
    origin: Vec3,
    direction: Vec3,
) -> anyhow::Result<Vec<(EntityId, f32)>> {
    Ok(elements_physics::intersection::raycast(
        world,
        Ray::new(origin, direction),
    ))
}
