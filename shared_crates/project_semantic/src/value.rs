use crate::{ItemId, ItemMap};

use super::{PrimitiveType, Type};
use ambient_project::Identifier;
use ambient_shared_types::{
    primitive_component_definitions, ProceduralMaterialHandle, ProceduralMeshHandle,
    ProceduralSamplerHandle, ProceduralTextureHandle,
};
use anyhow::Context;
use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

pub type EntityId = u128;

#[derive(Clone, PartialEq, Debug)]
pub enum ResolvedValue {
    Primitive(PrimitiveValue),
    Enum(ItemId<Type>, Identifier),
}
impl ResolvedValue {
    fn from_toml_value(value: &toml::Value, items: &ItemMap, id: ItemId<Type>) -> Self {
        match items.get_without_resolve(id).unwrap() {
            Type::Enum(e) => {
                let variant = value.as_str().unwrap();
                let variant = e
                    .members
                    .iter()
                    .find(|(name, _description)| name.as_ref() == variant)
                    .unwrap();
                Self::Enum(id, variant.0.clone())
            }
            ty => Self::Primitive(PrimitiveValue::from_toml_value(value, ty)),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ResolvableValue {
    Unresolved(toml::Value),
    Resolved(ResolvedValue),
}
impl ResolvableValue {
    pub(crate) fn resolve(&mut self, items: &ItemMap, id: ItemId<Type>) {
        if let Self::Unresolved(value) = self {
            *self = Self::Resolved(ResolvedValue::from_toml_value(value, items, id));
        }
    }
}

macro_rules! define_primitive_value {
    ($(($value:ident, $type:ty)),*) => {
        paste::paste! {
            #[derive(Debug, Clone, PartialEq)]
            pub enum PrimitiveValue {
                $(
                    $value($type),
                    [<Vec $value>](Vec<$type>),
                    [<Option $value>](Option<$type>),
                )*
            }
            $(
                impl From<$type> for PrimitiveValue {
                    fn from(value: $type) -> Self {
                        Self::$value(value)
                    }
                }
            )*
        }
    };
}

primitive_component_definitions!(define_primitive_value);

impl PrimitiveValue {
    pub fn from_toml_value(value: &toml::Value, ty: &Type) -> Self {
        match ty {
            Type::Primitive(pt) => Self::primitive_from_toml_value(value, *pt)
                .context("Failed to parse primitive value")
                .unwrap(),
            Type::Vec(_v) => todo!(),
            Type::Option(_o) => todo!(),
            Type::Enum(_) => unreachable!("Enum should be resolved"),
        }
    }

    pub fn primitive_from_toml_value(value: &toml::Value, ty: PrimitiveType) -> Option<Self> {
        pub(crate) fn as_array<T: Default + Copy, const N: usize>(
            value: &toml::Value,
            converter: impl Fn(&toml::Value) -> Option<T>,
        ) -> Option<[T; N]> {
            let arr = value.as_array().unwrap();
            assert_eq!(arr.len(), N);

            let mut result = [T::default(); N];
            for (i, v) in arr.iter().enumerate() {
                result[i] = converter(v)?;
            }
            Some(result)
        }

        let v = value;
        Some(match ty {
            PrimitiveType::Empty => Self::Empty(()),
            PrimitiveType::Bool => Self::Bool(v.as_bool()?),
            PrimitiveType::EntityId => Self::EntityId(EntityId::from_le_bytes(
                data_encoding::BASE64
                    .decode(v.as_str()?.as_bytes())
                    .unwrap()
                    .try_into()
                    .unwrap(),
            )),
            PrimitiveType::F32 => Self::F32(v.as_float()? as f32),
            PrimitiveType::F64 => Self::F64(v.as_float()?),
            PrimitiveType::Mat4 => Self::Mat4(Mat4::from_cols_array(&as_array(v, |v| {
                v.as_float().map(|v| v as f32)
            })?)),
            PrimitiveType::I32 => Self::I32(v.as_integer().map(|v| v as i32)?),
            PrimitiveType::Quat => Self::Quat(Quat::from_array(as_array(v, |v| {
                v.as_float().map(|v| v as f32)
            })?)),
            PrimitiveType::String => Self::String(v.as_str()?.to_string()),
            PrimitiveType::U8 => Self::U8(v.as_integer().map(|v| v as u8)?),
            PrimitiveType::U32 => Self::U32(v.as_integer().map(|v| v as u32)?),
            PrimitiveType::U64 => Self::U64(v.as_integer().map(|v| v as u64)?),
            PrimitiveType::Vec2 => Self::Vec2(Vec2::from_array(as_array(v, |v| {
                v.as_float().map(|v| v as f32)
            })?)),
            PrimitiveType::Vec3 => Self::Vec3(Vec3::from_array(as_array(v, |v| {
                v.as_float().map(|v| v as f32)
            })?)),
            PrimitiveType::Vec4 => Self::Vec4(Vec4::from_array(as_array(v, |v| {
                v.as_float().map(|v| v as f32)
            })?)),
            PrimitiveType::Uvec2 => Self::Uvec2(UVec2::from_array(as_array(v, |v| {
                v.as_integer().map(|v| v as u32)
            })?)),
            PrimitiveType::Uvec3 => Self::Uvec3(UVec3::from_array(as_array(v, |v| {
                v.as_integer().map(|v| v as u32)
            })?)),
            PrimitiveType::Uvec4 => Self::Uvec4(UVec4::from_array(as_array(v, |v| {
                v.as_integer().map(|v| v as u32)
            })?)),
            PrimitiveType::ProceduralMeshHandle => return None,
            PrimitiveType::ProceduralTextureHandle => return None,
            PrimitiveType::ProceduralSamplerHandle => return None,
            PrimitiveType::ProceduralMaterialHandle => return None,
        })
    }
}
