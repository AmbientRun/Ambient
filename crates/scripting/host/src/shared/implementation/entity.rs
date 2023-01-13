use crate::shared::host_guest_state::QueryStateMap;
use anyhow::Context;
use elements_core::transform::translation;
use elements_ecs::{
    query as ecs_query, with_component_registry, EntityId, QueryEvent, QueryState, World,
};
use elements_ecs::{Component, ComponentValue};
use glam::Vec3;
use slotmap::Key;

pub fn get_component_type<T: ComponentValue>(component_index: u64) -> Option<Component<T>> {
    with_component_registry(|r| r.get_by_index_type::<Component<T>>(component_index as usize))
}

pub fn get_component_index(id: &str) -> Option<usize> {
    with_component_registry(|r| Some(r.get_by_id(id)?.get_index()))
}

pub fn has_component(world: &World, entity_id: EntityId, index: usize) -> bool {
    world.has_component_index(entity_id, index)
}

pub fn remove_component(
    world: &mut World,
    entity_id: EntityId,
    index: usize,
) -> anyhow::Result<()> {
    let component = with_component_registry(|cr| cr.get_by_index(index).map(|c| c.clone_boxed()))
        .context("no component for index")?;
    Ok(world.remove_component(entity_id, component)?)
}

pub fn query(world: &mut World, index: u64) -> Vec<EntityId> {
    let component =
        match with_component_registry(|r| Some(r.get_by_index(index as usize)?.clone_boxed())) {
            Some(c) => c,
            None => return vec![],
        };

    elements_ecs::Query::new(elements_ecs::ArchetypeFilter::new().incl_ref(component.as_ref()))
        .iter(world, None)
        .map(|ea| ea.id())
        .collect()
}
pub fn query2(
    query_states: &mut QueryStateMap,
    components: impl Iterator<Item = u64> + Sync + Send,
    include: impl Iterator<Item = u64> + Sync + Send,
    exclude: impl Iterator<Item = u64> + Sync + Send,
    changed: impl Iterator<Item = u64> + Sync + Send,
    query_event: QueryEvent,
) -> anyhow::Result<u64> {
    fn get_components(
        registry: &elements_ecs::ComponentRegistry,
        components: impl Iterator<Item = u64> + Sync + Send,
    ) -> anyhow::Result<Vec<elements_ecs::PrimitiveComponent>> {
        components
            .map(|c| {
                registry
                    .get_primitive_component(c as usize)
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

    let mut query = elements_ecs::Query::new(elements_ecs::ArchetypeFilter::new());
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

pub fn in_area(world: &mut World, centre: Vec3, radius: f32) -> anyhow::Result<Vec<EntityId>> {
    Ok(ecs_query((translation(),))
        .iter(world, None)
        .filter_map(|(id, (pos,))| ((*pos - centre).length() < radius).then_some(id))
        .collect())
}

pub fn for_each_entity_in_radius(
    world: &mut World,
    position: Vec3,
    radius: f32,
    callback: impl Fn(EntityId) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let entities = in_area(world, position, radius)?;
    for id in entities {
        callback(id)?;
    }
    Ok(())
}
