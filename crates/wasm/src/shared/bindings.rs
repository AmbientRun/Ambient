use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use kiwi_ecs::{
    paste::paste, primitive_component_definitions, with_component_registry, Component,
    ComponentDesc, ComponentEntry, ECSError, EntityData, EntityId, World,
};
use kiwi_std::asset_url::ObjectRef;

use super::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::entity::get_component_type,
    interface::host,
};

pub type ComponentsParam<'a> = Vec<(u32, host::ComponentTypeParam<'a>)>;

use kiwi_ecs::PrimitiveComponentType as PCT;

macro_rules! define_component_types {
    ($(($value:ident, $type:ty)),*) => { paste! {
        fn read_primitive_component_from_world(
            world: &World,
            entity_id: EntityId,
            primitive_component: kiwi_ecs::PrimitiveComponent,
        ) -> Option<host::ComponentTypeResult> {
            use kiwi_ecs::PrimitiveComponentType as PCT;
            use host::{ComponentTypeResult as CTR, ComponentListTypeResult as CLTR, ComponentOptionTypeResult as COTR};

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
                PCT::$value            => CTR::[<Type $value>](get::<$type>(world, entity_id, c)?),
                PCT::[<Vec $value>]    => CTR::TypeList(CLTR::[<Type $value>](get::<Vec<$type>>(world, entity_id, c)?),),
                PCT::[<Option $value>] => CTR::TypeOption(COTR::[<Type $value>](get::<Option<$type>>(world, entity_id, c)?),),
                )*
            })
        }

        pub(crate) fn read_primitive_component_from_entity_accessor(
            world: &World,
            entity_accessor: &kiwi_ecs::EntityAccessor,
            primitive_component: kiwi_ecs::PrimitiveComponent,
        ) -> Option<host::ComponentTypeResult> {
            use kiwi_ecs::PrimitiveComponentType as PCT;
            use host::{ComponentTypeResult as CTR, ComponentListTypeResult as CLTR, ComponentOptionTypeResult as COTR};

            fn get<T: IntoBindgen + Clone + Send + Sync + 'static>(
                world: &World,
                entity_accessor: &kiwi_ecs::EntityAccessor,
                component: ComponentDesc,
            ) -> <T as IntoBindgen>::Item {
                entity_accessor.get(world, Component::<T>::new(component)).clone().into_bindgen()
            }

            let c = primitive_component.desc;
            Some(match primitive_component.ty {
                $(
                PCT::$value            => CTR::[<Type $value>](get::<$type>(world, entity_accessor, c).clone()),
                PCT::[<Vec $value>]    => CTR::TypeList(CLTR::[<Type $value>](get::<Vec<$type>>(world, entity_accessor, c).clone()),),
                PCT::[<Option $value>] => CTR::TypeOption(COTR::[<Type $value>](get::<Option<$type>>(world, entity_accessor, c).clone()),),
                )*
            })
        }

        pub(crate) fn read_component_from_world(
            world: &World,
            entity_id: EntityId,
            index: u32,
        ) -> Option<host::ComponentTypeResult> {
            let primitive_component = with_component_registry(|r| r.get_primitive_component(index))?;
            read_primitive_component_from_world(world, entity_id, primitive_component)
        }

        pub(crate) fn convert_entity_data_to_components(ed: &EntityData) -> Vec<(u32, host::ComponentTypeResult)> {
            use host::{
                ComponentListTypeResult as CLTR, ComponentOptionTypeResult as COTR,
                ComponentTypeResult as CTR,
            };

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
                            PCT::$value            => CTR::[<Type $value>](get::<$type>(cu)?),
                            PCT::[<Vec $value>]    => CTR::TypeList(CLTR::[<Type $value>](get::<Vec<$type>>(cu)?),),
                            PCT::[<Option $value>] => CTR::TypeOption(COTR::[<Type $value>](get::<Option<$type>>(cu)?),),
                            )*
                        };

                        Some((index, value))
                    })
                    .collect()
            })
        }

        // todo: find a nice efficient abstraction to tie these three functions together
        pub(crate) fn convert_components_to_entity_data(
            components: ComponentsParam<'_>,
        ) -> EntityData {
            use host::{
                ComponentListTypeParam as CLTP, ComponentOptionTypeParam as COTP,
                ComponentTypeParam as CTP,
            };
            with_component_registry(|cr| {
                components
                    .into_iter()
                    .flat_map(|(index, value)| {
                        let primitive_component = cr.get_primitive_component(index)?;
                        let c = primitive_component.desc;

                        match (primitive_component.ty, value) {
                            $(
                            (PCT::$value, CTP::[<Type $value>](v))                              => Some(ComponentEntry::from_raw_parts(c, v.from_bindgen())),
                            (PCT::[<Vec $value>], CTP::TypeList(CLTP::[<Type $value>](v)))      => Some(ComponentEntry::from_raw_parts(c, v.from_bindgen())),
                            (PCT::[<Option $value>], CTP::TypeOption(COTP::[<Type $value>](v))) => Some(ComponentEntry::from_raw_parts(c, v.from_bindgen()))
                            ),*,
                            _ => None,
                        }
                    })
                    .collect()
            })
        }

        pub(crate) fn add_component(
            world: &mut World,
            entity_id: EntityId,
            index: u32,
            value: host::ComponentTypeParam<'_>,
        ) -> Result<(), ECSError> {
            match value {
                $(
                host::ComponentTypeParam::[<Type $value >](value) => {
                    if let Some(component) = get_component_type::<$type>(index) {
                        world.add_component(entity_id, component, value.from_bindgen())?;
                    }
                }
                host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Vec<$type>>(index) {
                        world.add_component(entity_id, component, value.from_bindgen())?;
                    }
                }
                host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::[<Type $value >](value)) => {
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
            entity_id: EntityId,
            index: u32,
            value: host::ComponentTypeParam<'_>,
        ) -> Result<(), ECSError> {
            match value {
                $(
                host::ComponentTypeParam::[<Type $value >](value) => {
                    if let Some(component) = get_component_type::<$type>(index) {
                        world.set(entity_id, component, value.from_bindgen())?;
                    }
                }
                host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Vec<$type>>(index) {
                        world.set(entity_id, component, value.from_bindgen())?;
                    }
                }
                host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::[<Type $value >](value)) => {
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
