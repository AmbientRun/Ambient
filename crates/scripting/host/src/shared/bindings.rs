use std::any::TypeId;

use elements_ecs::{
    paste::paste, with_component_registry, Component, ComponentDesc, ComponentEntry, EntityData,
    EntityId, EntityUid, World,
};
use elements_std::asset_url::ObjectRef;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use once_cell::sync::Lazy;

use super::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::entity::get_component_type,
    interface as sif,
};

pub type ComponentsParam<'a> = Vec<(u32, sif::ComponentTypeParam<'a>)>;

use elements_ecs::PrimitiveComponentType as PCT;

macro_rules! define_component_types {
    ($(($type:ty, $value:ident)),*) => { paste! {
        pub(crate) static SUPPORTED_COMPONENT_TYPES: Lazy<Vec< (TypeId, &str) >> = Lazy::new(|| vec![$(
            (TypeId::of::<$type>(), stringify!($type)),
            (TypeId::of::<Vec<$type>>(), stringify!(Vec<$type>)),
            (TypeId::of::<Option<$type>>(), stringify!(Option<$type>))
        ),*]);

        fn read_primitive_component_from_world(
            world: &World,
            entity_id: EntityId,
            primitive_component: elements_ecs::PrimitiveComponent,
        ) -> Option<sif::ComponentTypeResult> {
            use elements_ecs::PrimitiveComponentType as PCT;
            use sif::{ComponentTypeResult as CTR, ComponentListTypeResult as CLTR, ComponentOptionTypeResult as COTR};

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
                _ => unreachable!(),
            })
        }

        pub(crate) fn read_primitive_component_from_entity_accessor(
            world: &World,
            entity_accessor: &elements_ecs::EntityAccessor,
            primitive_component: elements_ecs::PrimitiveComponent,
        ) -> Option<sif::ComponentTypeResult> {
            use elements_ecs::PrimitiveComponentType as PCT;
            use sif::{ComponentTypeResult as CTR, ComponentListTypeResult as CLTR, ComponentOptionTypeResult as COTR};

            fn get<T: IntoBindgen + Clone + Send + Sync + 'static>(
                world: &World,
                entity_accessor: &elements_ecs::EntityAccessor,
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
                _ => unreachable!(),
            })
        }

        pub(crate) fn read_component_from_world(
            world: &World,
            entity_id: EntityId,
            index: u32,
        ) -> Option<sif::ComponentTypeResult> {
            let primitive_component = with_component_registry(|r| r.get_primitive_component(index))?;
            read_primitive_component_from_world(world, entity_id, primitive_component)
        }

        pub(crate) fn convert_entity_data_to_components(ed: &EntityData) -> Vec<(u32, sif::ComponentTypeResult)> {
            use sif::{
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
                            component: ComponentDesc,
                        ) -> Option<<T as IntoBindgen>::Item> {
                            Some(
                                entry
                                    .downcast_cloned::<T>()
                                    .into_bindgen(),
                            )
                        }

            let c = primitive_component.desc;
                        let value = match primitive_component.ty {
                            $(
                            PCT::$value            => CTR::[<Type $value>](get::<$type>(cu, c)?),
                            PCT::[<Vec $value>]    => CTR::TypeList(CLTR::[<Type $value>](get::<Vec<$type>>(cu, c)?),),
                            PCT::[<Option $value>] => CTR::TypeOption(COTR::[<Type $value>](get::<Option<$type>>(cu, c)?),),
                            )*
                            _ => unreachable!(),
                        };

                        Some((index, value))
                    })
                    .collect()
            })
        }

        pub(crate) fn convert_components_to_entity_data(
            components: ComponentsParam<'_>,
        ) -> EntityData {
            use sif::{
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

        pub(crate) fn write_component(
            world: &mut World,
            entity_id: EntityId,
            index: u32,
            value: sif::ComponentTypeParam<'_>,
        ) {
            match value {
                $(
                sif::ComponentTypeParam::[<Type $value >](value) => {
                    if let Some(component) = get_component_type::<$type>(index) {
                        world.add_component(entity_id, component, value.from_bindgen()).unwrap();
                    }
                }
                sif::ComponentTypeParam::TypeList(sif::ComponentListTypeParam::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Vec<$type>>(index) {
                        world.add_component(entity_id, component, value.from_bindgen()).unwrap();
                    }
                }
                sif::ComponentTypeParam::TypeOption(sif::ComponentOptionTypeParam::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Option<$type>>(index) {
                        world.add_component(entity_id, component, value.from_bindgen()).unwrap();
                    }
                }
                ) *
            }
        }
    }};
}

define_component_types!(
    ((), Empty),
    (bool, Bool),
    (EntityId, EntityId),
    (f32, F32),
    (f64, F64),
    (Mat4, Mat4),
    (i32, I32),
    (Quat, Quat),
    (String, String),
    (u32, U32),
    (u64, U64),
    (Vec2, Vec2),
    (Vec3, Vec3),
    (Vec4, Vec4),
    (ObjectRef, ObjectRef),
    (EntityUid, EntityUid)
);
