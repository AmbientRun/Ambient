use ambient_ecs::{
    with_component_registry, Component, ComponentDesc, ComponentEntry, ComponentSet,
    ComponentValue, Entity, EntityId, PrimitiveComponentType as PCT, QueryEvent, QueryState, World,
};
use ambient_shared_types::primitive_component_definitions;
use ambient_shared_types::{
    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
    ProceduralTextureHandle,
};
use anyhow::Context;
use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
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

pub fn get_component_type<T: ComponentValue>(component_index: u32) -> Option<Component<T>> {
    let desc = with_component_registry(|r| r.get_by_index(component_index))?;

    Some(Component::new(desc))
}

macro_rules! define_component_types {
    ($(($value:ident, $type:ty)),*) => { paste! {
        fn read_primitive_component_from_world(
            world: &World,
            entity_id: EntityId,
            primitive_component: ambient_ecs::PrimitiveComponent,
        ) -> Option<wit::component::Value> {
            use ambient_ecs::PrimitiveComponentType as PCT;
            use wit::component::{Value as V, VecValue as VV, OptionValue as OV};

            fn get<T: IntoBindgen + Clone + Send + Sync + 'static>(
                world: &World,
                id: EntityId,
                component: ComponentDesc,
            ) -> Option<<T as IntoBindgen>::Item> {
                Some(world.get_cloned(id, Component::<T>::new(component)).ok()?.into_bindgen())
            }

            let c = primitive_component.desc;
            Some(match primitive_component.ty {
                $(
                PCT::$value            => V::[<Type $value>](get::<$type>(world, entity_id, c)?),
                PCT::[<Vec $value>]    => V::TypeVec(VV::[<Type $value>](get::<Vec<$type>>(world, entity_id, c)?),),
                PCT::[<Option $value>] => V::TypeOption(OV::[<Type $value>](get::<Option<$type>>(world, entity_id, c)?),),
                )*
            })
        }

        pub(crate) fn read_primitive_component_from_entity_accessor(
            world: &World,
            entity_accessor: &ambient_ecs::EntityAccessor,
            primitive_component: ambient_ecs::PrimitiveComponent,
        ) -> Option<wit::component::Value> {
            use ambient_ecs::PrimitiveComponentType as PCT;
            use wit::component::{Value as V, VecValue as VV, OptionValue as OV};

            fn get<T: IntoBindgen + Clone + Send + Sync + 'static>(
                world: &World,
                entity_accessor: &ambient_ecs::EntityAccessor,
                component: ComponentDesc,
            ) -> <T as IntoBindgen>::Item {
                entity_accessor.get(world, Component::<T>::new(component)).clone().into_bindgen()
            }

            let c = primitive_component.desc;
            Some(match primitive_component.ty {
                $(
                PCT::$value            => V::[<Type $value>](get::<$type>(world, entity_accessor, c).clone()),
                PCT::[<Vec $value>]    => V::TypeVec(VV::[<Type $value>](get::<Vec<$type>>(world, entity_accessor, c).clone()),),
                PCT::[<Option $value>] => V::TypeOption(OV::[<Type $value>](get::<Option<$type>>(world, entity_accessor, c).clone()),),
                )*
            })
        }

        pub(crate) fn get_component(
            world: &World,
            entity_id: wit::types::EntityId,
            index: u32,
        ) -> anyhow::Result<Option<wit::component::Value>> {
            let Some(primitive_component) = with_component_registry(|r| r.get_primitive_component(index)) else { return Ok(None); };
            Ok(read_primitive_component_from_world(world, entity_id.from_bindgen(), primitive_component))
        }

        #[allow(dead_code)]
        pub(crate) fn convert_entity_data_to_components(ed: &Entity) -> Vec<(u32, wit::component::Value)> {
            use wit::component::{VecValue as VV, OptionValue as OV, Value as V};

            with_component_registry(|cr| {
                ed.iter()
                    .flat_map(|cu| {
                        let index = cu.index();
                        let primitive_component = cr.get_primitive_component(index)?;
                        fn get<T: IntoBindgen + Clone + Send + Sync + 'static>(
                            entry: &ComponentEntry,
                        ) -> Option<<T as IntoBindgen>::Item> {
                            Some(
                                entry
                                    .downcast_cloned::<T>()
                                    .into_bindgen(),
                            )
                        }

                        let value = match primitive_component.ty {
                            $(
                            PCT::$value            => V::[<Type $value>](get::<$type>(cu)?),
                            PCT::[<Vec $value>]    => V::TypeVec(VV::[<Type $value>](get::<Vec<$type>>(cu)?),),
                            PCT::[<Option $value>] => V::TypeOption(OV::[<Type $value>](get::<Option<$type>>(cu)?),),
                            )*
                        };

                        Some((index, value))
                    })
                    .collect()
            })
        }

        // todo: find a nice efficient abstraction to tie these three functions together
        pub(crate) fn convert_components_to_entity_data(
            components: wit::entity::EntityData,
        ) -> Entity {
            use wit::component::{VecValue as VV, OptionValue as OV, Value as V};
            with_component_registry(|cr| {
                components
                    .into_iter()
                    .flat_map(|(index, value)| {
                        let primitive_component = cr.get_primitive_component(index)?;
                        let c = primitive_component.desc;

                        match (primitive_component.ty, value) {
                            $(
                            (PCT::$value, V::[<Type $value>](v))                              => Some(ComponentEntry::from_raw_parts(c, v.from_bindgen())),
                            (PCT::[<Vec $value>], V::TypeVec(VV::[<Type $value>](v)))      => Some(ComponentEntry::from_raw_parts(c, v.from_bindgen())),
                            (PCT::[<Option $value>], V::TypeOption(OV::[<Type $value>](v))) => Some(ComponentEntry::from_raw_parts(c, v.from_bindgen()))
                            ),*,
                            _ => None,
                        }
                    })
                    .collect()
            })
        }

        pub(crate) fn add_component(
            world: &mut World,
            entity_id: wit::types::EntityId,
            index: u32,
            value: wit::component::Value,
        ) -> anyhow::Result<()> {
            use wit::component::{VecValue as VV, OptionValue as OV, Value as V};

            let entity_id = entity_id.from_bindgen();
            match value {
                $(
                V::[<Type $value >](value) => {
                    if let Some(component) = get_component_type::<$type>(index) {
                        world.add_component(entity_id, component, value.from_bindgen())?;
                    }
                }
                V::TypeVec(VV::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Vec<$type>>(index) {
                        world.add_component(entity_id, component, value.from_bindgen())?;
                    }
                }
                V::TypeOption(OV::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Option<$type>>(index) {
                        world.add_component(entity_id, component, value.from_bindgen())?;
                    }
                }
                ) *
            }

            Ok(())
        }

        pub(crate) fn set_component(
            world: &mut World,
            entity_id: wit::types::EntityId,
            index: u32,
            value: wit::component::Value,
        ) -> anyhow::Result<()> {
            use wit::component::{VecValue as VV, OptionValue as OV, Value as V};

            let entity_id = entity_id.from_bindgen();
            match value {
                $(
                V::[<Type $value >](value) => {
                    if let Some(component) = get_component_type::<$type>(index) {
                        world.set(entity_id, component, value.from_bindgen())?;
                    }
                }
                V::TypeVec(VV::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Vec<$type>>(index) {
                        world.set(entity_id, component, value.from_bindgen())?;
                    }
                }
                V::TypeOption(OV::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Option<$type>>(index) {
                        world.set(entity_id, component, value.from_bindgen())?;
                    }
                }
                ) *
            }

            Ok(())
        }
    }};
}

primitive_component_definitions!(define_component_types);

pub(crate) fn add_components(
    world: &mut World,
    entity_id: wit::types::EntityId,
    data: wit::entity::EntityData,
) -> anyhow::Result<()> {
    Ok(world.add_components(
        entity_id.from_bindgen(),
        convert_components_to_entity_data(data),
    )?)
}

pub(crate) fn set_components(
    world: &mut World,
    entity_id: wit::types::EntityId,
    data: wit::entity::EntityData,
) -> anyhow::Result<()> {
    Ok(world.set_components(
        entity_id.from_bindgen(),
        convert_components_to_entity_data(data),
    )?)
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
            get_components(cr, &query.include)?,
            get_components(cr, &query.exclude)?,
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
                    .map(|pc| {
                        read_primitive_component_from_entity_accessor(world, &ea, pc.clone())
                            .unwrap()
                    })
                    .collect(),
            )
        })
        .collect_vec();
    query_states.get_mut(key).unwrap().1 = query_state;

    Ok(result)
}
