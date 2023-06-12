use crate::{
    global::{
        EntityId, Mat4, ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
        ProceduralTextureHandle, Quat, Vec2, Vec3, Vec4,
    },
    internal::{
        component::Component,
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
};
use ambient_shared_types::primitive_component_definitions;
use glam::{UVec2, UVec3, UVec4};
use std::time::Duration;

#[doc(hidden)]
pub fn get_component<T>(id: &str) -> Component<T> {
    Component::new(wit::component::get_index(id).unwrap())
}

/// Implemented by all types that can be used as values in components.
pub trait SupportedValue
where
    Self: Sized,
{
    #[doc(hidden)]
    fn from_result(result: wit::component::Value) -> Option<Self>;

    #[doc(hidden)]
    fn into_result(self) -> wit::component::Value;
}

macro_rules! define_component_types {
    ($(($value:ident, $type:ty)),*) => { paste::paste! {
        $(
        impl SupportedValue for $type {
            fn from_result(result: wit::component::Value) -> Option<Self> {
                match result {
                    wit::component::Value::[< Type $value >](v) => Some(v.from_bindgen()),
                    _ => None,
                }
            }

            fn into_result(self) -> wit::component::Value {
                wit::component::Value::[< Type $value >](self.into_bindgen())
            }
        }
        impl SupportedValue for Vec<$type> {
            fn from_result(result: wit::component::Value) -> Option<Self> {
                match result {
                    wit::component::Value::TypeVec(wit::component::VecValue::[< Type $value >](v)) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
                    _ => None,
                }
            }

            fn into_result(self) -> wit::component::Value {
                wit::component::Value::TypeVec(wit::component::VecValue::[< Type $value >](self.into_bindgen()))
            }
        }
        impl SupportedValue for Option<$type> {
            fn from_result(result: wit::component::Value) -> Option<Self> {
                match result {
                    wit::component::Value::TypeOption(wit::component::OptionValue::[< Type $value >](v)) => Some(v.from_bindgen()),
                    _ => None,
                }
            }

            fn into_result(self) -> wit::component::Value {
                wit::component::Value::TypeOption(wit::component::OptionValue::[< Type $value >](self.into_bindgen()))
            }
        }
        ) *
    } };
}

primitive_component_definitions!(define_component_types);
