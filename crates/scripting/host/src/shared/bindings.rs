use std::any::TypeId;

use elements_ecs::{
    paste::paste, with_component_registry, Component, ComponentUnit, EntityData, EntityId,
    EntityUid, World,
};
use elements_std::asset_url::ObjectRef;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use once_cell::sync::Lazy;

use super::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::entity::get_component_type,
    interface as sif,
};

pub type ComponentsParam<'a> = Vec<(u64, sif::ComponentTypeParam<'a>)>;

macro_rules! define_component_types {
    ($(($type:ty, $value:ident)),*) => { paste! {
        pub static SUPPORTED_COMPONENT_TYPES: Lazy<Vec<(TypeId, &'static str)>> = Lazy::new(|| vec![$(
            (TypeId::of::<Component<$type>>(), stringify!($type)),
            (TypeId::of::<Component<Vec<$type>>>(), stringify!(Vec<$type>)),
            (TypeId::of::<Component<Option<$type>>>(), stringify!(Option<$type>))
        ),*]);

        pub(crate) fn read_primitive_component_from_world(
            world: &World,
            entity_id: EntityId,
            primitive_component: elements_ecs::PrimitiveComponent,
        ) -> Option<sif::ComponentTypeResult> {
            use elements_ecs::PrimitiveComponent as PC;
            use sif::{ComponentTypeResult as CTR, ComponentListTypeResult as CLTR, ComponentOptionTypeResult as COTR};

            fn get<T: IntoBindgen + Clone + Send + Sync + 'static>(
                world: &World,
                id: EntityId,
                component: Component<T>,
            ) -> Option<<T as IntoBindgen>::Item> {
                Some(world.get_cloned(id, component).ok()?.into_bindgen())
            }

            Some(match primitive_component {
                $(
                PC::$value(c) => CTR::[<Type $value>](get(world, entity_id, c)?),
                PC::[<Vec $value>](c) => CTR::TypeList(CLTR::[<Type $value>](get(world, entity_id, c)?),),
                PC::[<Option $value>](c) => CTR::TypeOption(COTR::[<Type $value>](get(world, entity_id, c)?),)
                ),*
            })
        }

        pub(crate) fn read_primitive_component_from_entity_accessor(
            world: &World,
            entity_accessor: &elements_ecs::EntityAccessor,
            primitive_component: elements_ecs::PrimitiveComponent,
        ) -> Option<sif::ComponentTypeResult> {
            use elements_ecs::PrimitiveComponent as PC;
            use sif::{ComponentTypeResult as CTR, ComponentListTypeResult as CLTR, ComponentOptionTypeResult as COTR};

            fn get<T: IntoBindgen + Clone + Send + Sync + 'static>(
                world: &World,
                entity_accessor: &elements_ecs::EntityAccessor,
                component: Component<T>,
            ) -> <T as IntoBindgen>::Item {
                entity_accessor.get(world, component).clone().into_bindgen()
            }

            Some(match primitive_component {
                $(
                PC::$value(c) => CTR::[<Type $value>](get(world, entity_accessor, c).clone()),
                PC::[<Vec $value>](c) => CTR::TypeList(CLTR::[<Type $value>](get(world, entity_accessor, c).clone()),),
                PC::[<Option $value>](c) => CTR::TypeOption(COTR::[<Type $value>](get(world, entity_accessor, c).clone()),)
                ),*
            })
        }

        pub(crate) fn read_component_from_world(
            world: &World,
            entity_id: EntityId,
            index: u64,
        ) -> Option<sif::ComponentTypeResult> {
            let primitive_component = with_component_registry(|r| r.get_primitive_component(index as usize))?;
            read_primitive_component_from_world(world, entity_id, primitive_component)
        }

        pub(crate) fn convert_entity_data_to_components(ed: &EntityData) -> Vec<(u64, sif::ComponentTypeResult)> {
            use elements_ecs::PrimitiveComponent as PC;
            use sif::{
                ComponentListTypeResult as CLTR, ComponentOptionTypeResult as COTR,
                ComponentTypeResult as CTR,
            };

            with_component_registry(|cr| {
                ed.iter()
                    .flat_map(|cu| {
                        let index = cu.get_component_index();
                        let primitive_component = cr.get_primitive_component(index)?;
                        fn get<T: IntoBindgen + Clone + Send + Sync + 'static>(
                            component_unit: &ComponentUnit,
                            component: Component<T>,
                        ) -> Option<<T as IntoBindgen>::Item> {
                            Some(
                                component_unit
                                    .as_component_value(component)
                                    .cloned()?
                                    .into_bindgen(),
                            )
                        }
                        let value = match primitive_component {
                            $(
                            PC::$value(c) => CTR::[<Type $value>](get(cu, c)?),
                            PC::[<Vec $value>](c) => CTR::TypeList(CLTR::[<Type $value>](get(cu, c)?),),
                            PC::[<Option $value>](c) => CTR::TypeOption(COTR::[<Type $value>](get(cu, c)?),)
                            ),*
                        };
                        Some((index as u64, value))
                    })
                    .collect()
            })
        }

        pub(crate) fn convert_components_to_entity_data(
            components: ComponentsParam<'_>,
        ) -> EntityData {
            use elements_ecs::PrimitiveComponent as PC;
            use sif::{
                ComponentListTypeParam as CLTP, ComponentOptionTypeParam as COTP,
                ComponentTypeParam as CTP,
            };
            with_component_registry(|cr| {
                components
                    .into_iter()
                    .flat_map(|(index, value)| {
                        let primitive_component = cr.get_primitive_component(index as usize)?;
                        match (primitive_component, value) {
                            $(
                            (PC::$value(c), CTP::[<Type $value>](v)) => Some(ComponentUnit::new(c, v.from_bindgen())),
                            (PC::[<Vec $value>](c), CTP::TypeList(CLTP::[<Type $value>](v))) => Some(ComponentUnit::new(c, v.from_bindgen())),
                            (PC::[<Option $value>](c), CTP::TypeOption(COTP::[<Type $value>](v))) => Some(ComponentUnit::new(c, v.from_bindgen()))
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
            index: u64,
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
