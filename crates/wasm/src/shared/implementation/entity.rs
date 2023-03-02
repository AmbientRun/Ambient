use ambient_animation::{animation_controller, AnimationController};
use ambient_core::transform::translation;
use ambient_ecs::{
    query as ecs_query, with_component_registry, Component, ComponentValue, Entity, EntityId,
    QueryEvent, QueryState, World,
};
use anyhow::Context;
use glam::Vec3;
use slotmap::Key;

use crate::shared::bindings::QueryStateMap;

pub fn spawn(world: &mut World, data: Entity) -> EntityId {
    data.spawn(world)
}

pub fn despawn(world: &mut World, entity: EntityId) -> Option<EntityId> {
    world.despawn(entity).map(|_ed| entity)
}

pub fn set_animation_controller(
    world: &mut World,
    entity: EntityId,
    controller: AnimationController,
) -> anyhow::Result<()> {
    Ok(world.add_component(entity, animation_controller(), controller)?)
}

pub fn get_component_type<T: ComponentValue>(component_index: u32) -> Option<Component<T>> {
    let desc = with_component_registry(|r| r.get_by_index(component_index))?;

    Some(Component::new(desc))
}

pub fn get_component_index(id: &str) -> Option<u32> {
    with_component_registry(|r| Some(r.get_by_path(id)?.index()))
}

pub fn has_component(world: &World, entity_id: EntityId, index: u32) -> bool {
    world.has_component_index(entity_id, index)
}

pub fn remove_component(world: &mut World, entity_id: EntityId, index: u32) -> anyhow::Result<()> {
    let desc =
        with_component_registry(|cr| cr.get_by_index(index)).context("no component for index")?;

    Ok(world.remove_component(entity_id, desc)?)
}

pub fn get_all(world: &mut World, index: u32) -> Vec<EntityId> {
    let desc = match with_component_registry(|r| r.get_by_index(index)) {
        Some(c) => c,
        None => return vec![],
    };

    ambient_ecs::Query::new(ambient_ecs::ArchetypeFilter::new().incl_ref(desc))
        .iter(world, None)
        .map(|ea| ea.id())
        .collect()
}
pub fn query(
    query_states: &mut QueryStateMap,
    components: &[u32],
    include: &[u32],
    exclude: &[u32],
    changed: &[u32],
    query_event: QueryEvent,
) -> anyhow::Result<u64> {
    fn get_components(
        registry: &ambient_ecs::ComponentRegistry,
        components: &[u32],
    ) -> anyhow::Result<Vec<ambient_ecs::PrimitiveComponent>> {
        components
            .iter()
            .map(|c| {
                registry
                    .get_primitive_component(*c)
                    .context("no primitive component")
            })
            .collect()
    }

    let (components, include, exclude, changed) = with_component_registry(|cr| {
        anyhow::Ok((
            get_components(cr, components)?,
            get_components(cr, include)?,
            get_components(cr, exclude)?,
            get_components(cr, changed)?,
        ))
    })?;

    let mut query = ambient_ecs::Query::new(ambient_ecs::ArchetypeFilter::new());
    query.event = query_event;
    for component in &components {
        query = query.incl_ref(component.as_component());
    }
    for component in include {
        query = query.incl_ref(component.as_component());
    }
    for component in exclude {
        query = query.excl_ref(component.as_component());
    }
    for component in changed {
        query = query.optional_changed_ref(component.as_component());
    }

    Ok(query_states
        .insert((query, QueryState::new(), components))
        .data()
        .as_ffi())
}

pub fn resources(world: &World) -> EntityId {
    world.resource_entity()
}

pub fn in_area(world: &mut World, centre: Vec3, radius: f32) -> anyhow::Result<Vec<EntityId>> {
    Ok(ecs_query((translation(),))
        .iter(world, None)
        .filter_map(|(id, (pos,))| ((*pos - centre).length() < radius).then_some(id))
        .collect())
}
