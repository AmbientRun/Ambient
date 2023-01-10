use elements_animation::{animation_controller, AnimationController};
use elements_ecs::{uid, EntityData, EntityId, EntityUid, World};
use elements_object::{fire_spawn_by_url, MultiEntityUID, SpawnConfig};
use elements_physics::helpers as eph;
use glam::{Mat4, Quat, Vec3};

pub fn spawn(world: &mut World, data: EntityData) -> EntityUid {
    let uid = EntityUid::create();
    data.set(elements_ecs::uid(), uid.clone()).spawn(world);
    uid
}

pub fn spawn_template(
    world: &mut World,
    object: String,
    position: Vec3,
    rotation: Option<Quat>,
    scale: Option<Vec3>,
) -> EntityUid {
    let uid = MultiEntityUID::new();
    fire_spawn_by_url(
        world,
        object,
        SpawnConfig::new(
            uid.clone(),
            position,
            rotation.unwrap_or(Quat::IDENTITY),
            scale.unwrap_or(Vec3::ONE),
        ),
        None,
    );
    // TODO(fred): This will only return the first entity spawned. Need async spawn to return all
    uid.get_uid(0)
}

pub fn despawn(world: &mut World, entity: EntityId) -> Option<EntityUid> {
    world.despawn(entity).and_then(|ed| ed.get_cloned(uid()))
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
