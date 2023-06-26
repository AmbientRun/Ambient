use std::collections::HashSet;

use ambient_core::transform::{local_to_world, translation};
use ambient_ecs::{query as ecs_query, with_component_registry, EntityId, World};

use ambient_network::ServerWorldExt;

use anyhow::Context;
use glam::Mat4;

use super::{
    super::{
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
    component::{convert_components_to_entity_data, convert_entity_data_to_components},
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
) -> anyhow::Result<Option<wit::entity::EntityData>> {
    let id = id.from_bindgen();
    spawned_entities.remove(&id);
    Ok(world
        .despawn(id)
        .map(|e| convert_entity_data_to_components(&e)))
}

pub fn get_transforms_relative_to(
    world: &World,
    list: Vec<wit::types::EntityId>,
    origin: wit::types::EntityId,
) -> anyhow::Result<Vec<wit::types::Mat4>> {
    let origin_id = origin.from_bindgen();

    let transform = world
        .get(origin_id, local_to_world())
        .unwrap_or(Mat4::IDENTITY)
        .inverse();

    let mut result = Vec::with_capacity(list.len());

    for entity in list {
        let entity_id = entity.from_bindgen();
        let relative = transform
            * world
                .get(entity_id, local_to_world())
                .unwrap_or(Mat4::IDENTITY);
        result.push(relative.into_bindgen());
    }

    Ok(result)
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
