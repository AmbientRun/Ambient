use ambient_ecs::{
    with_component_registry, Component, ComponentEntry, ComponentSet, ComponentValue, Entity,
    EntityAccessor, EntityId, Enum, PrimitiveComponent, PrimitiveComponentType as PCT, QueryEvent,
    QueryState, World,
};
use ambient_shared_types::primitive_component_definitions;
use ambient_shared_types::{
    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
    ProceduralTextureHandle,
};
use anyhow::Context;
use glam::{IVec2, IVec3, IVec4, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use itertools::Itertools;
use paste::paste;
use slotmap::Key;
use std::time::Duration;

use crate::shared::bindings::QueryStateMap;

use super::super::{
    conversion::{FromBindgen, IntoBindgen},
    wit,
};

pub fn get_index(id: String) -> anyhow::Result<Option<u32>> {
    Ok(with_component_registry(|r| {
        Some(r.get_by_path(&id)?.index())
    }))
}

pub fn get_id(index: u32) -> anyhow::Result<Option<String>> {
    Ok(with_component_registry(|r| {
        Some(r.get_by_index(index)?.path())
    }))
}

pub fn get_component_type<T: ComponentValue>(component_index: u32) -> Option<Component<T>> {
    let desc = with_component_registry(|r| r.get_by_index(component_index))?;

    Some(Component::new(desc))
}

trait WitValueVisitor<Context> {
    fn visit<T: ComponentValue>(
        &mut self,
        ctx: Context,
        component: Component<T>,
        value: T,
    ) -> anyhow::Result<()>;
}
trait HostValueVisitor<Context> {
    fn visit<T: ComponentValue + IntoBindgen>(
        &mut self,
        ctx: Context,
        component: Component<T>,
    ) -> anyhow::Result<Option<T>>;
}

macro_rules! define_component_types {
    ($(($value:ident, $type:ty)),*) => { paste! {
        /// Given a WIT value and a component index, do something with the
        /// typed component and value.
        fn visit_wit_value<Context>(
            ctx: Context,
            index: u32,
            value: wit::component::Value,
            mut operation: impl WitValueVisitor<Context>,
        ) -> anyhow::Result<()> {
            use wit::component::{OptionValue as OV, Value as V, VecValue as VV};
            match value {
                $(
                V::[<Type $value >](value) => {
                    if let Some(component) = get_component_type::<$type>(index) {
                        operation.visit(ctx, component, value.from_bindgen())?;
                    }
                }
                V::TypeVec(VV::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Vec<$type>>(index) {
                        operation.visit(ctx, component, value.from_bindgen())?;
                    }
                }
                V::TypeOption(OV::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Option<$type>>(index) {
                        operation.visit(ctx, component, value.from_bindgen())?;
                    }
                }
                ) *
            }

            Ok(())
        }

        /// Given a primitive component and a host context, extract the
        /// value from the host context and return it as a WIT value.
        fn visit_host_value<Context>(
            ctx: Context,
            primitive_component: ambient_ecs::PrimitiveComponent,
            mut operation: impl HostValueVisitor<Context>,
        ) -> anyhow::Result<Option<wit::component::Value>> {
            use wit::component::{OptionValue as OV, Value as V, VecValue as VV};

            Ok(match primitive_component.ty {
                $(
                PCT::$value            => {
                    let component = Component::<$type>::new(primitive_component.desc);
                    operation.visit(ctx, component)?.map(|v| V::[<Type $value>](v.into_bindgen()))
                },
                PCT::[<Vec $value>]    => {
                    let component = Component::<Vec<$type>>::new(primitive_component.desc);
                    operation.visit(ctx, component)?.map(|v| V::TypeVec(VV::[<Type $value>](v.into_bindgen())))
                },
                PCT::[<Option $value>] => {
                    let component = Component::<Option<$type>>::new(primitive_component.desc);
                    operation.visit(ctx, component)?.map(|v| V::TypeOption(OV::[<Type $value>](v.into_bindgen())))
                },
                )*
            })
        }
    }};
}

primitive_component_definitions!(define_component_types);

fn enum_value_to_entry(index: u32, value: &wit::component::Value) -> Option<ComponentEntry> {
    with_component_registry(|cr| {
        if let wit::component::Value::TypeU32(value) = *value {
            let desc = cr.get_by_index(index)?;
            (desc.attribute::<Enum>()?.from_u32)(desc, value)
        } else {
            None
        }
    })
}

pub(crate) fn add_component(
    world: &mut World,
    id: wit::entity::EntityId,
    index: u32,
    value: wit::component::Value,
) -> anyhow::Result<()> {
    if let Some(entry) = enum_value_to_entry(index, &value) {
        world.add_entry(id.from_bindgen(), entry)?;
        return Ok(());
    }

    struct WorldAdd(EntityId);
    impl<'a> WitValueVisitor<&'a mut World> for WorldAdd {
        fn visit<T: ComponentValue>(
            &mut self,
            ctx: &'a mut World,
            component: Component<T>,
            value: T,
        ) -> anyhow::Result<()> {
            ctx.add_component(self.0, component, value)?;
            Ok(())
        }
    }
    visit_wit_value(world, index, value, WorldAdd(id.from_bindgen()))
}

pub(crate) fn set_component(
    world: &mut World,
    id: wit::entity::EntityId,
    index: u32,
    value: wit::component::Value,
) -> anyhow::Result<()> {
    if let Some(entry) = enum_value_to_entry(index, &value) {
        world.set_entry(id.from_bindgen(), entry)?;
        return Ok(());
    }

    struct WorldSet(EntityId);
    impl<'a> WitValueVisitor<&'a mut World> for WorldSet {
        fn visit<T: ComponentValue>(
            &mut self,
            ctx: &'a mut World,
            component: Component<T>,
            value: T,
        ) -> anyhow::Result<()> {
            ctx.set(self.0, component, value)?;
            Ok(())
        }
    }
    visit_wit_value(world, index, value, WorldSet(id.from_bindgen()))
}

pub(crate) fn set_components(
    world: &mut World,
    id: wit::entity::EntityId,
    data: wit::entity::EntityData,
) -> anyhow::Result<()> {
    Ok(world.set_components(id.from_bindgen(), wit_entity_to_host_entity(data)?)?)
}

pub(crate) fn add_components(
    world: &mut World,
    id: wit::entity::EntityId,
    data: wit::entity::EntityData,
) -> anyhow::Result<()> {
    Ok(world.add_components(id.from_bindgen(), wit_entity_to_host_entity(data)?)?)
}

pub(crate) fn get_component(
    world: &World,
    id: wit::entity::EntityId,
    index: u32,
) -> anyhow::Result<Option<wit::component::Value>> {
    let primitive_component = with_component_registry(|cr| cr.get_primitive_component(index))
        .with_context(|| format!("the component {index} does not exist"))?;

    get_component_entity_accessor(
        world,
        &EntityAccessor::World {
            id: id.from_bindgen(),
        },
        primitive_component,
    )
}

fn get_component_entity_accessor<'a>(
    world: &'a World,
    ea: &'a EntityAccessor,
    primitive_component: PrimitiveComponent,
) -> anyhow::Result<Option<wit::component::Value>> {
    struct EntityAccessorGet<'a>(&'a EntityAccessor);
    impl<'a> HostValueVisitor<&'a World> for EntityAccessorGet<'a> {
        fn visit<T: ComponentValue>(
            &mut self,
            ctx: &'a World,
            component: Component<T>,
        ) -> anyhow::Result<Option<T>> {
            Ok(self.0.get_optional(ctx, component).cloned())
        }
    }
    visit_host_value(world, primitive_component, EntityAccessorGet(ea))
}

pub(crate) fn get_components(
    world: &World,
    id: wit::entity::EntityId,
    indices: Vec<u32>,
) -> anyhow::Result<wit::entity::EntityData> {
    let primitive_components: Vec<_> = with_component_registry(|cr| {
        indices
            .into_iter()
            .flat_map(|index| Some((index, cr.get_primitive_component(index)?)))
            .collect()
    });

    let id = id.from_bindgen();
    let entity_accessor = EntityAccessor::World { id };
    let mut entity = wit::entity::EntityData::new();
    for (index, pc) in primitive_components {
        if let Some(v) = get_component_entity_accessor(world, &entity_accessor, pc)? {
            entity.push((index, v));
        }
    }
    Ok(entity)
}

pub(crate) fn get_all_components(
    world: &World,
    id: wit::entity::EntityId,
) -> anyhow::Result<wit::entity::EntityData> {
    get_components(
        world,
        id,
        world
            .get_components(id.from_bindgen())?
            .into_iter()
            .map(|d| d.index())
            .collect(),
    )
}

pub(crate) fn wit_entity_to_host_entity(
    wit_entity: wit::entity::EntityData,
) -> anyhow::Result<Entity> {
    struct EntityProducer;
    impl<'a> WitValueVisitor<&'a mut Entity> for EntityProducer {
        fn visit<T: ComponentValue>(
            &mut self,
            ctx: &'a mut Entity,
            component: Component<T>,
            value: T,
        ) -> anyhow::Result<()> {
            ctx.set(component, value);
            Ok(())
        }
    }

    let mut entity = Entity::new();
    for (index, value) in wit_entity {
        if let Some(entry) = enum_value_to_entry(index, &value) {
            entity.set_entry(entry);
            continue;
        }

        visit_wit_value(&mut entity, index, value, EntityProducer)?;
    }
    Ok(entity)
}

pub(crate) fn host_entity_to_wit_entity(entity: Entity) -> anyhow::Result<wit::entity::EntityData> {
    struct EntityExtractor;
    impl<'a> HostValueVisitor<&'a Entity> for EntityExtractor {
        fn visit<T: ComponentValue>(
            &mut self,
            ctx: &'a Entity,
            component: Component<T>,
        ) -> anyhow::Result<Option<T>> {
            Ok(ctx.get_cloned(component))
        }
    }

    let components: Vec<_> = with_component_registry(|cr| {
        entity
            .iter()
            .flat_map(|ce| cr.get_primitive_component(ce.index()))
            .collect()
    });
    let mut wit_entity = wit::entity::EntityData::new();
    for component in components {
        let index = component.desc.index();
        if let Some(value) = visit_host_value(&entity, component, EntityExtractor)? {
            wit_entity.push((index, value));
        }
    }
    Ok(wit_entity)
}

pub fn has_component(
    world: &World,
    entity_id: wit::types::EntityId,
    index: u32,
) -> anyhow::Result<bool> {
    Ok(world.has_component_index(entity_id.from_bindgen(), index))
}

pub fn has_components(
    world: &World,
    entity_id: wit::types::EntityId,
    components: Vec<u32>,
) -> anyhow::Result<bool> {
    let mut set = ComponentSet::new();
    for idx in components {
        set.insert_by_index(idx as usize);
    }
    Ok(world.has_components(entity_id.from_bindgen(), &set))
}

pub fn remove_component(
    world: &mut World,
    entity_id: wit::types::EntityId,
    index: u32,
) -> anyhow::Result<()> {
    let desc =
        with_component_registry(|cr| cr.get_by_index(index)).context("no component for index")?;

    Ok(world.remove_component(entity_id.from_bindgen(), desc)?)
}

pub fn remove_components(
    world: &mut World,
    entity_id: wit::types::EntityId,
    components: Vec<u32>,
) -> anyhow::Result<()> {
    let components = with_component_registry(|cr| {
        components
            .into_iter()
            .flat_map(|idx| cr.get_by_index(idx))
            .collect()
    });
    Ok(world.remove_components(entity_id.from_bindgen(), components)?)
}

pub fn query(
    query_states: &mut QueryStateMap,
    query: wit::component::QueryBuild,
    query_event: wit::component::QueryEvent,
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
            get_components(cr, &query.components)?,
            get_components(cr, &query.includes)?,
            get_components(cr, &query.excludes)?,
            get_components(cr, &query.changed)?,
        ))
    })?;

    let mut query = ambient_ecs::Query::new(ambient_ecs::ArchetypeFilter::new());
    query.event = match query_event {
        wit::component::QueryEvent::Frame => QueryEvent::Frame,
        wit::component::QueryEvent::Spawn => QueryEvent::Spawned,
        wit::component::QueryEvent::Despawn => QueryEvent::Despawned,
    };
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

pub fn query_eval(
    world: &World,
    query_states: &mut QueryStateMap,
    query_index: u64,
) -> anyhow::Result<Vec<(wit::types::EntityId, Vec<wit::component::Value>)>> {
    let key = slotmap::DefaultKey::from(slotmap::KeyData::from_ffi(query_index));

    let (query, query_state, primitive_components) =
        query_states.get(key).context("no query state for key")?;

    let mut query_state = query_state.clone();
    let result = query
        .iter(world, Some(&mut query_state))
        .map(|ea| {
            (
                ea.id().into_bindgen(),
                primitive_components
                    .iter()
                    .flat_map(|pc| get_component_entity_accessor(world, &ea, pc.clone()).unwrap())
                    .collect(),
            )
        })
        .collect_vec();
    query_states.get_mut(key).unwrap().1 = query_state;

    Ok(result)
}
