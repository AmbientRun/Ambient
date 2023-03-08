use std::collections::HashSet;

use ambient_ecs::{
    paste::paste, primitive_component_definitions, with_component_registry, Component,
    ComponentDesc, ComponentEntry, ECSError, Entity as EntityData, EntityId, PrimitiveComponent,
    PrimitiveComponentType as PCT, Query, QueryState, World,
};

use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

use super::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::entity::get_component_type,
    wit,
};

pub type ComponentsParam = Vec<(u32, wit::component::ValueResult)>;

pub type QueryStateMap =
    slotmap::SlotMap<slotmap::DefaultKey, (Query, QueryState, Vec<PrimitiveComponent>)>;

#[derive(Clone, Default)]
pub struct BindingsBase {
    pub spawned_entities: HashSet<EntityId>,
    pub subscribed_events: HashSet<String>,
    pub query_states: QueryStateMap,
}

pub trait BindingsBound:
    wit::types::Host
    + wit::component::Host
    + wit::entity::Host
    + wit::player::Host
    + wit::physics::Host
    + wit::event::Host
    + wit::asset::Host
    + Clone
    + Sync
    + Send
{
    fn base(&self) -> &BindingsBase;
    fn base_mut(&mut self) -> &mut BindingsBase;

    fn set_world(&mut self, world: &mut World);
    fn clear_world(&mut self);
}

#[derive(Clone)]
pub struct WorldRef(*mut World);
impl Default for WorldRef {
    fn default() -> Self {
        Self::new()
    }
}
impl WorldRef {
    const fn new() -> Self {
        WorldRef(std::ptr::null_mut())
    }
    pub unsafe fn world(&self) -> &World {
        unsafe { self.0.as_ref().unwrap() }
    }
    pub unsafe fn world_mut(&mut self) -> &mut World {
        unsafe { self.0.as_mut().unwrap() }
    }
    pub unsafe fn set_world(&mut self, world: &mut World) {
        self.0 = world;
    }
    pub unsafe fn clear_world(&mut self) {
        self.0 = std::ptr::null_mut();
    }
}
unsafe impl Send for WorldRef {}
unsafe impl Sync for WorldRef {}

macro_rules! define_component_types {
    ($(($value:ident, $type:ty)),*) => { paste! {
        fn read_primitive_component_from_world(
            world: &World,
            entity_id: EntityId,
            primitive_component: ambient_ecs::PrimitiveComponent,
        ) -> Option<wit::component::ValueResult> {
            use ambient_ecs::PrimitiveComponentType as PCT;
            use wit::component::{ValueResult as VR, VecValueResult as VVR, OptionValueResult as OVR};

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
                PCT::$value            => VR::[<Type $value>](get::<$type>(world, entity_id, c)?),
                PCT::[<Vec $value>]    => VR::TypeVec(VVR::[<Type $value>](get::<Vec<$type>>(world, entity_id, c)?),),
                PCT::[<Option $value>] => VR::TypeOption(OVR::[<Type $value>](get::<Option<$type>>(world, entity_id, c)?),),
                )*
            })
        }

        pub(crate) fn read_primitive_component_from_entity_accessor(
            world: &World,
            entity_accessor: &ambient_ecs::EntityAccessor,
            primitive_component: ambient_ecs::PrimitiveComponent,
        ) -> Option<wit::component::ValueResult> {
            use ambient_ecs::PrimitiveComponentType as PCT;
            use wit::component::{ValueResult as VR, VecValueResult as VVR, OptionValueResult as OVR};

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
                PCT::$value            => VR::[<Type $value>](get::<$type>(world, entity_accessor, c).clone()),
                PCT::[<Vec $value>]    => VR::TypeVec(VVR::[<Type $value>](get::<Vec<$type>>(world, entity_accessor, c).clone()),),
                PCT::[<Option $value>] => VR::TypeOption(OVR::[<Type $value>](get::<Option<$type>>(world, entity_accessor, c).clone()),),
                )*
            })
        }

        pub(crate) fn read_component_from_world(
            world: &World,
            entity_id: EntityId,
            index: u32,
        ) -> Option<wit::component::ValueResult> {
            let primitive_component = with_component_registry(|r| r.get_primitive_component(index))?;
            read_primitive_component_from_world(world, entity_id, primitive_component)
        }

        pub(crate) fn convert_entity_data_to_components(ed: &EntityData) -> Vec<(u32, wit::component::ValueResult)> {
            use wit::component::{
                VecValueResult as VVR, OptionValueResult as OVR,
                ValueResult as VR,
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
                            PCT::$value            => VR::[<Type $value>](get::<$type>(cu)?),
                            PCT::[<Vec $value>]    => VR::TypeVec(VVR::[<Type $value>](get::<Vec<$type>>(cu)?),),
                            PCT::[<Option $value>] => VR::TypeOption(OVR::[<Type $value>](get::<Option<$type>>(cu)?),),
                            )*
                        };

                        Some((index, value))
                    })
                    .collect()
            })
        }

        // todo: find a nice efficient abstraction to tie these three functions together
        pub(crate) fn convert_components_to_entity_data(
            components: ComponentsParam,
        ) -> EntityData {
            use wit::component::{
                VecValueResult as VVR, OptionValueResult as OVR,
                ValueResult as VR,
            };
            with_component_registry(|cr| {
                components
                    .into_iter()
                    .flat_map(|(index, value)| {
                        let primitive_component = cr.get_primitive_component(index)?;
                        let c = primitive_component.desc;

                        match (primitive_component.ty, value) {
                            $(
                            (PCT::$value, VR::[<Type $value>](v))                              => Some(ComponentEntry::from_raw_parts(c, v.from_bindgen())),
                            (PCT::[<Vec $value>], VR::TypeVec(VVR::[<Type $value>](v)))      => Some(ComponentEntry::from_raw_parts(c, v.from_bindgen())),
                            (PCT::[<Option $value>], VR::TypeOption(OVR::[<Type $value>](v))) => Some(ComponentEntry::from_raw_parts(c, v.from_bindgen()))
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
            value: wit::component::ValueResult,
        ) -> Result<(), ECSError> {
            use wit::component::{
                VecValueResult as VVR, OptionValueResult as OVR,
                ValueResult as VR,
            };
            match value {
                $(
                VR::[<Type $value >](value) => {
                    if let Some(component) = get_component_type::<$type>(index) {
                        world.add_component(entity_id, component, value.from_bindgen())?;
                    }
                }
                VR::TypeVec(VVR::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Vec<$type>>(index) {
                        world.add_component(entity_id, component, value.from_bindgen())?;
                    }
                }
                VR::TypeOption(OVR::[<Type $value >](value)) => {
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
            value: wit::component::ValueResult,
        ) -> Result<(), ECSError> {
            use wit::component::{
                VecValueResult as VVR, OptionValueResult as OVR,
                ValueResult as VR,
            };
            match value {
                $(
                VR::[<Type $value >](value) => {
                    if let Some(component) = get_component_type::<$type>(index) {
                        world.set(entity_id, component, value.from_bindgen())?;
                    }
                }
                VR::TypeVec(VVR::[<Type $value >](value)) => {
                    if let Some(component) = get_component_type::<Vec<$type>>(index) {
                        world.set(entity_id, component, value.from_bindgen())?;
                    }
                }
                VR::TypeOption(OVR::[<Type $value >](value)) => {
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
