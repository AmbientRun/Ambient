use std::collections::HashSet;

use ambient_animation::{
    animation_binder_mask, animation_binder_weights, animation_controller, animation_stack,
};
use ambient_core::transform::{local_to_world, translation};
use ambient_ecs::{query as ecs_query, with_component_registry, EntityId, World};

use ambient_model::animation_binder;
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

pub fn set_animation_action_stack(
    world: &mut World,
    entity: wit::types::EntityId,
    stack: Vec<wit::entity::AnimationActionStack>,
    transition_duration: f32,
) -> anyhow::Result<()> {
    if let Ok(current) = world.get_mut(entity.from_bindgen(), animation_stack()) {
        if !current.is_empty() && transition_duration > 0.0 {
            use ambient_animation::AnimationActionStack::*;
            current.push(Transition {
                weight: 0.0,
                duration: transition_duration,
            });
        } else {
            current.clear();
        }

        current.extend(stack.into_iter().map(|x| x.from_bindgen()));
        return Ok(());
    }

    Ok(world.add_component(
        entity.from_bindgen(),
        animation_stack(),
        stack.into_iter().map(|x| x.from_bindgen()).collect(),
    )?)
}

pub fn set_animation_binder_mask(
    world: &mut World,
    entity: wit::types::EntityId,
    mask: Vec<String>,
) -> anyhow::Result<()> {
    Ok(world.add_component(entity.from_bindgen(), animation_binder_mask(), mask)?)
}

pub fn set_animation_binder_weights(
    world: &mut World,
    entity: wit::types::EntityId,
    index: u32,
    mask: Vec<f32>,
) -> anyhow::Result<()> {
    let entity_id = entity.from_bindgen();
    let index = index as usize;

    if let Ok(weights) = world.get_mut(entity_id, animation_binder_weights()) {
        if weights.len() <= index {
            weights.resize(index + 1, Vec::default());
        }
        weights[index] = mask;
        Ok(())
    } else {
        let mut weights = vec![Vec::default(); index + 1];
        weights[index] = mask;
        Ok(world.add_component(entity.from_bindgen(), animation_binder_weights(), weights)?)
    }
}

pub fn get_animation_binder_mask_entities(
    world: &mut World,
    entity: wit::types::EntityId,
) -> anyhow::Result<Vec<wit::types::EntityId>> {
    let entity_id = entity.from_bindgen();

    let binder: &std::collections::HashMap<String, EntityId> = world
        .get_ref(entity_id, animation_binder())
        .context("missing animation_binder")?;

    if let Ok(mask) = world.get_ref(entity_id, animation_binder_mask()) {
        if !mask.is_empty() {
            return Ok(mask
                .iter()
                .map(|x| binder.get(x).unwrap_or(&EntityId::null()).into_bindgen())
                .collect())
        }
    }

    let mut mask: Vec<String> = binder.keys().cloned().collect();
    mask.sort();
    let result = mask
        .iter()
        .map(|x| binder.get(x).unwrap_or(&EntityId::null()).into_bindgen())
        .collect();
    world.add_component(entity_id, animation_binder_mask(), mask)?;
    Ok(result)
}

pub fn get_animation_binder_mask(
    world: &mut World,
    entity: wit::types::EntityId,
) -> anyhow::Result<Vec<String>> {
    let entity_id = entity.from_bindgen();
    if let Ok(mask) = world.get_ref(entity_id, animation_binder_mask()) {
        if !mask.is_empty() {
            return Ok(mask.clone());
        }
    }

    if let Ok(binder) = world.get_ref(entity_id, animation_binder()) {
        let mut mask: Vec<String> = binder.keys().cloned().collect();
        mask.sort();
        world.add_component(entity_id, animation_binder_mask(), mask.clone())?;
        return Ok(mask);
    }
    Ok(Vec::new())
}

pub fn play_animation_action_index(
    world: &mut World,
    entity: wit::types::EntityId,
    index: u32,
    speed: f32,
    transition_duration: f32,
) -> anyhow::Result<()> {
    let entity_id = entity.from_bindgen();

    use ambient_animation::AnimationActionStack::*;
    if let Ok(stack) = world.get_mut(entity_id, animation_stack()) {
        if transition_duration <= 0.0 {
            stack.clear();
        } else if !stack.is_empty() {
            stack.push(Transition {
                weight: 0.0,
                duration: transition_duration,
            });
        }

        stack.push(Start {
            action_index: index,
            speed,
        });
    } else {
        world.add_component(
            entity_id,
            animation_stack(),
            vec![Sample {
                action_index: index,
            }],
        )?;
    }

    Ok(())
}

pub fn get_transforms_relative_to(
    world: &World,
    list: Vec<wit::types::EntityId>,
    origin: wit::types::EntityId,
) -> anyhow::Result<Vec<wit::types::Mat4>> {
    let origin_id = origin.from_bindgen();

    let transform = world
        .get(origin_id, local_to_world())
        .unwrap_or_else(|_| Mat4::IDENTITY)
        .inverse();

    let mut result = Vec::with_capacity(list.len());

    for entity in list {
        let entity_id = entity.from_bindgen();
        let relative = transform
            * world
                .get(entity_id, local_to_world())
                .unwrap_or_else(|_| Mat4::IDENTITY);
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
