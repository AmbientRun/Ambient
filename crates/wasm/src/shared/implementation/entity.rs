use std::collections::HashSet;

use ambient_animation::{animation_controller, AnimationActionTime};
use ambient_core::transform::translation;
use ambient_ecs::{query as ecs_query, with_component_registry, EntityId, World};

use ambient_network::ServerWorldExt;

use anyhow::Context;

use super::{
    super::{
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
    component::convert_components_to_entity_data,
};

pub fn spawn(
    world: &mut World,
    spawned_entities: &mut HashSet<EntityId>,
    data: wit::entity::EntityData,
) -> anyhow::Result<wit::types::EntityId> {
    let id = convert_components_to_entity_data(data).spawn(world);
    spawned_entities.insert(id);
    Ok(id.into_bindgen())
}

pub fn despawn(
    world: &mut World,
    spawned_entities: &mut HashSet<EntityId>,
    id: wit::types::EntityId,
) -> anyhow::Result<bool> {
    let id = id.from_bindgen();
    spawned_entities.remove(&id);
    Ok(world.despawn(id).is_some())
}

pub fn set_animation_controller(
    world: &mut World,
    entity: wit::types::EntityId,
    controller: wit::entity::AnimationController,
) -> anyhow::Result<()> {
    Ok(world.add_component(
        entity.from_bindgen(),
        animation_controller(),
        controller.from_bindgen(),
    )?)
}

pub fn set_animation_blend(
    world: &mut World,
    entity: wit::types::EntityId,
    weights: &[f32],
    times: &[f32],
    absolute_time: bool,
) -> anyhow::Result<()> {
    let controller = world.get_mut(entity.from_bindgen(), animation_controller())?;
    for (action, weight) in controller.actions.iter_mut().zip(weights.iter()) {
        action.weight = *weight;
    }

    if absolute_time {
        for (action, time) in controller.actions.iter_mut().zip(times.iter()) {
            action.time = AnimationActionTime::Absolute { time: *time };
        }
    } else {
        for (action, time) in controller.actions.iter_mut().zip(times.iter()) {
            action.time = AnimationActionTime::Percentage { percentage: *time }
        }
    }
    Ok(())
}

pub fn exists(world: &World, entity: wit::types::EntityId) -> anyhow::Result<bool> {
    Ok(world.exists(entity.from_bindgen()))
}

pub fn resources(world: &World) -> anyhow::Result<wit::types::EntityId> {
    Ok(world.resource_entity().into_bindgen())
}

pub fn synchronized_resources(world: &World) -> anyhow::Result<wit::types::EntityId> {
    Ok(world
        .synced_resource_entity()
        .context("no entity")?
        .into_bindgen())
}

pub fn persisted_resources(world: &World) -> anyhow::Result<wit::types::EntityId> {
    Ok(world
        .persisted_resource_entity()
        .context("no entity")?
        .into_bindgen())
}

pub fn in_area(
    world: &mut World,
    centre: wit::types::Vec3,
    radius: f32,
) -> anyhow::Result<Vec<wit::types::EntityId>> {
    let centre = centre.from_bindgen();
    Ok(ecs_query((translation(),))
        .iter(world, None)
        .filter_map(|(id, (pos,))| ((*pos - centre).length() < radius).then_some(id))
        .map(|id| id.into_bindgen())
        .collect())
}

pub fn get_all(world: &mut World, index: u32) -> anyhow::Result<Vec<wit::types::EntityId>> {
    let desc = match with_component_registry(|r| r.get_by_index(index)) {
        Some(c) => c,
        None => return Ok(vec![]),
    };

    Ok(
        ambient_ecs::Query::new(ambient_ecs::ArchetypeFilter::new().incl_ref(desc))
            .iter(world, None)
            .map(|ea| ea.id().into_bindgen())
            .collect(),
    )
}
