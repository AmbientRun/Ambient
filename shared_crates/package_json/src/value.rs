use std::{fmt, time::Duration};

use ambient_primitive_component_definitions::primitive_component_definitions;
use serde::{Deserialize, Serialize};

use crate::{Identifier, ItemId, Type};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Scalar(ScalarValue),
    Vec(Vec<ScalarValue>),
    Option(Option<ScalarValue>),
    Enum(EnumValue),
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Scalar(v) => fmt::Display::fmt(v, f),
            Value::Vec(v) => fmt::Debug::fmt(v, f),
            Value::Option(v) => fmt::Debug::fmt(v, f),
            Value::Enum(v) => write!(f, "{}", v.member),
        }
    }
}
macro_rules! define_scalar_value {
    ($(($value:ident, $type:ty)),*) => {
        #[derive(Serialize, Deserialize, Clone, PartialEq)]
        #[serde(tag = "type", content = "value")]
        pub enum ScalarValue {
            $(
                $value($type),
            )*
        }
        impl fmt::Debug for ScalarValue {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$value(value) => fmt::Debug::fmt(value, f),
                    )*
                }
            }
        }
        impl fmt::Display for ScalarValue {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$value(value) => fmt::Debug::fmt(value, f),
                    )*
                }
            }
        }

    };
}
primitive_component_definitions!(define_scalar_value);

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EnumValue {
    pub ty: ItemId<Type>,
    pub member: Identifier,
}

pub type EntityId = String;
pub type Mat4 = [f32; 16];
pub type Quat = [f32; 4];
pub type Vec2 = [f32; 2];
pub type Vec3 = [f32; 3];
pub type Vec4 = [f32; 4];
pub type UVec2 = [u32; 2];
pub type UVec3 = [u32; 3];
pub type UVec4 = [u32; 4];
pub type IVec2 = [i32; 2];
pub type IVec3 = [i32; 3];
pub type IVec4 = [i32; 4];
pub type ProceduralMeshHandle = String;
pub type ProceduralTextureHandle = String;
pub type ProceduralSamplerHandle = String;
pub type ProceduralMaterialHandle = String;
