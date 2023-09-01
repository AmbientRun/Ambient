use crate::{
    global::{
        EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle, ProceduralMeshHandle,
        ProceduralSamplerHandle, ProceduralTextureHandle, Quat, UVec2, UVec3, UVec4, Vec2, Vec3,
        Vec4,
    },
    internal::{
        component::Component,
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
};
use ambient_shared_types::primitive_component_definitions;
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

    /// Converts a `ComponentValue` into a `Self`.
    fn from_value(value: ComponentValue) -> Option<Self>;
    /// Converts a `Self` into a `ComponentValue`.
    fn into_value(self) -> ComponentValue;
}

/// Implemented by all types that can be used as referenceable values in components.
pub trait SupportedValueRef
where
    Self: SupportedValue,
{
    /// Converts a `&ComponentValue` into a `&Self`.
    fn from_value_ref(value: &ComponentValue) -> Option<&Self>;
}

macro_rules! define_component_types {
    ($(($value:ident, $type:ty)),*) => { paste::paste! {
        /// A value that can be stored in a component.
        #[derive(Clone, Debug, PartialEq)]
        #[allow(missing_docs)]
        pub enum ComponentValue {
            $(
                [<$value>]($type),
            )*
            Vec(ComponentVecValue),
            Option(ComponentOptionValue),
        }
        impl IntoBindgen for ComponentValue {
            type Item = wit::component::Value;
            fn into_bindgen(self) -> Self::Item {
                match self {
                    $(
                        Self::[<$value>](v) => wit::component::Value::[< Type $value >](v.into_bindgen()),
                    )*
                    Self::Vec(v) => wit::component::Value::TypeVec(v.into_bindgen()),
                    Self::Option(v) => wit::component::Value::TypeOption(v.into_bindgen()),
                }
            }
        }
        impl FromBindgen for wit::component::Value {
            type Item = ComponentValue;
            fn from_bindgen(self) -> Self::Item {
                match self {
                    $(
                        wit::component::Value::[< Type $value >](v) => ComponentValue::[<$value>](v.from_bindgen()),
                    )*
                    wit::component::Value::TypeVec(v) => ComponentValue::Vec(v.from_bindgen()),
                    wit::component::Value::TypeOption(v) => ComponentValue::Option(v.from_bindgen()),
                }
            }
        }

        /// A vector value that can be stored in a component.
        #[derive(Clone, Debug, PartialEq)]
        #[allow(missing_docs)]
        pub enum ComponentVecValue {
            $(
                [<$value>](Vec<$type>),
            )*
        }
        impl IntoBindgen for ComponentVecValue {
            type Item = wit::component::VecValue;
            fn into_bindgen(self) -> Self::Item {
                match self {
                    $(
                        Self::[<$value>](v) => wit::component::VecValue::[< Type $value >](v.into_bindgen()),
                    )*
                }
            }
        }
        impl FromBindgen for wit::component::VecValue {
            type Item = ComponentVecValue;
            fn from_bindgen(self) -> Self::Item {
                match self {
                    $(
                        wit::component::VecValue::[< Type $value >](v) => ComponentVecValue::[<$value>](v.from_bindgen()),
                    )*
                }
            }
        }

        /// An option value that can be stored in a component.
        #[derive(Clone, Debug, PartialEq)]
        #[allow(missing_docs)]
        pub enum ComponentOptionValue {
            $(
                [<$value>](Option<$type>),
            )*
        }
        impl IntoBindgen for ComponentOptionValue {
            type Item = wit::component::OptionValue;
            fn into_bindgen(self) -> Self::Item {
                match self {
                    $(
                        Self::[<$value>](v) => wit::component::OptionValue::[< Type $value >](v.into_bindgen()),
                    )*
                }
            }
        }
        impl FromBindgen for wit::component::OptionValue {
            type Item = ComponentOptionValue;
            fn from_bindgen(self) -> Self::Item {
                match self {
                    $(
                        wit::component::OptionValue::[< Type $value >](v) => ComponentOptionValue::[<$value>](v.from_bindgen()),
                    )*
                }
            }
        }

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

            fn from_value(value: ComponentValue) -> Option<Self> {
                match value {
                    ComponentValue::[<$value>](v) => Some(v),
                    _ => None,
                }
            }
            fn into_value(self) -> ComponentValue {
                ComponentValue::[<$value>](self)
            }
        }
        impl SupportedValueRef for $type {
            fn from_value_ref(value: &ComponentValue) -> Option<&Self> {
                match value {
                    ComponentValue::[<$value>](v) => Some(v),
                    _ => None,
                }
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

            fn from_value(value: ComponentValue) -> Option<Self> {
                match value {
                    ComponentValue::Vec(ComponentVecValue::[<$value>](v)) => Some(v),
                    _ => None,
                }
            }

            fn into_value(self) -> ComponentValue {
                ComponentValue::Vec(ComponentVecValue::[<$value>](self))
            }
        }
        impl SupportedValueRef for Vec<$type> {
            fn from_value_ref(value: &ComponentValue) -> Option<&Self> {
                match value {
                    ComponentValue::Vec(ComponentVecValue::[<$value>](v)) => Some(v),
                    _ => None,
                }
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

            fn from_value(value: ComponentValue) -> Option<Self> {
                match value {
                    ComponentValue::Option(ComponentOptionValue::[<$value>](v)) => Some(v),
                    _ => None,
                }
            }

            fn into_value(self) -> ComponentValue {
                ComponentValue::Option(ComponentOptionValue::[<$value>](self))
            }
        }
        impl SupportedValueRef for Option<$type> {
            fn from_value_ref(value: &ComponentValue) -> Option<&Self> {
                match value {
                    ComponentValue::Option(ComponentOptionValue::[<$value>](v)) => Some(v),
                    _ => None,
                }
            }
        }
        ) *
    } };
}

primitive_component_definitions!(define_component_types);
