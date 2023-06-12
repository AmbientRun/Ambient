use crate::{ItemId, ItemMap, TypeInner};

use super::{PrimitiveType, Type};
use ambient_project::Identifier;
use ambient_shared_types::{
    primitive_component_definitions, ProceduralMaterialHandle, ProceduralMeshHandle,
    ProceduralSamplerHandle, ProceduralTextureHandle,
};
use anyhow::Context as AnyhowContext;
use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use std::time::Duration;

pub type EntityId = u128;

#[derive(Clone, PartialEq, Debug)]
pub enum ResolvedValue {
    Primitive(PrimitiveValue),
    Enum(ItemId<Type>, Identifier),
}
impl ResolvedValue {
    fn from_toml_value(
        value: &toml::Value,
        items: &ItemMap,
        id: ItemId<Type>,
    ) -> anyhow::Result<Self> {
        let ty = &*items.get(id)?;
        Ok(match &ty.inner {
            TypeInner::Enum(e) => {
                let variant = value.as_str().with_context(|| {
                    format!("Expected string for enum variant, got {:?}", value)
                })?;

                let variant = e
                    .members
                    .iter()
                    .find(|(name, _description)| name.as_ref() == variant)
                    .with_context(|| {
                        format!(
                            "Expected enum variant to be one of {:?}, got {:?}",
                            e.members, variant
                        )
                    })?;

                Self::Enum(id, variant.0.clone())
            }
            _ => Self::Primitive(PrimitiveValue::from_toml_value(value, ty)?),
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ResolvableValue {
    Unresolved(toml::Value),
    Resolved(ResolvedValue),
}
impl ResolvableValue {
    pub(crate) fn resolve(&mut self, items: &ItemMap, id: ItemId<Type>) -> anyhow::Result<()> {
        if let Self::Unresolved(value) = self {
            *self = Self::Resolved(ResolvedValue::from_toml_value(value, items, id)?);
        }
        Ok(())
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
    pub(crate) fn from_toml_value(value: &toml::Value, ty: &Type) -> anyhow::Result<Self> {
        Ok(match ty.inner {
            TypeInner::Primitive(pt) => Self::primitive_from_toml_value(value, pt)?
                .with_context(|| format!("Failed to parse TOML value {:?} as {:?}", value, pt))?,
            TypeInner::Vec(_v) => todo!("We don't support vecs yet"),
            TypeInner::Option(_o) => todo!("We don't support options yet"),
            TypeInner::Enum(_) => unreachable!("Enum should be resolved"),
        })
    }

    pub fn primitive_from_toml_value(
        value: &toml::Value,
        ty: PrimitiveType,
    ) -> anyhow::Result<Option<Self>> {
        fn as_bool(v: &toml::Value) -> anyhow::Result<bool> {
            v.as_bool()
                .with_context(|| format!("Expected bool, got {:?}", v))
        }

        fn as_integer(v: &toml::Value) -> anyhow::Result<i64> {
            v.as_integer()
                .with_context(|| format!("Expected integer, got {:?}", v))
        }

        fn as_str(v: &toml::Value) -> anyhow::Result<&str> {
            v.as_str()
                .with_context(|| format!("Expected string, got {:?}", v))
        }

        fn as_array<T: Default + Copy, const N: usize>(
            value: &toml::Value,
            converter: impl Fn(&toml::Value) -> anyhow::Result<T>,
        ) -> anyhow::Result<[T; N]> {
            let arr = value
                .as_array()
                .with_context(|| format!("Expected array, got {:?}", value))?;

            assert_eq!(arr.len(), N);

            let mut result = [T::default(); N];
            for (i, v) in arr.iter().enumerate() {
                result[i] = converter(v)?;
            }
            Ok(result)
        }

        fn as_float(v: &toml::Value) -> anyhow::Result<f64> {
            v.as_float()
                .with_context(|| format!("Expected float, got {:?}", v))
        }

        let v = value;
        Ok(Some(match ty {
            PrimitiveType::Empty => Self::Empty(()),
            PrimitiveType::Bool => Self::Bool(as_bool(v)?),
            PrimitiveType::EntityId => {
                let value = as_str(v)?;
                let bytes = data_encoding::BASE64
                    .decode(value.as_bytes())
                    .with_context(|| format!("Failed to decode Base64 for entity id {value:?}"))?;
                let bytes = bytes.as_slice().try_into().with_context(|| {
                    format!("Failed to convert decoded Base64 bytes {bytes:?} to entity id")
                })?;
                Self::EntityId(EntityId::from_le_bytes(bytes))
            }
            PrimitiveType::F32 => Self::F32(as_float(v)? as f32),
            PrimitiveType::F64 => Self::F64(as_float(v)?),
            PrimitiveType::Mat4 => Self::Mat4(Mat4::from_cols_array(&as_array(v, |v| {
                Ok(as_float(v)? as f32)
            })?)),
            PrimitiveType::I32 => Self::I32(as_integer(v)? as i32),
            PrimitiveType::Quat => {
                Self::Quat(Quat::from_array(as_array(v, |v| Ok(as_float(v)? as f32))?))
            }
            PrimitiveType::String => Self::String(as_str(v)?.to_string()),
            PrimitiveType::U8 => Self::U8(as_integer(v)? as u8),
            PrimitiveType::U32 => Self::U32(as_integer(v)? as u32),
            PrimitiveType::U64 => Self::U64(as_integer(v)? as u64),
            PrimitiveType::Vec2 => {
                Self::Vec2(Vec2::from_array(as_array(v, |v| Ok(as_float(v)? as f32))?))
            }
            PrimitiveType::Vec3 => {
                Self::Vec3(Vec3::from_array(as_array(v, |v| Ok(as_float(v)? as f32))?))
            }
            PrimitiveType::Vec4 => {
                Self::Vec4(Vec4::from_array(as_array(v, |v| Ok(as_float(v)? as f32))?))
            }
            PrimitiveType::Uvec2 => Self::Uvec2(UVec2::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as u32)
            })?)),
            PrimitiveType::Uvec3 => Self::Uvec3(UVec3::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as u32)
            })?)),
            PrimitiveType::Uvec4 => Self::Uvec4(UVec4::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as u32)
            })?)),
            PrimitiveType::Duration => return Ok(None),
            PrimitiveType::ProceduralMeshHandle => return Ok(None),
            PrimitiveType::ProceduralTextureHandle => return Ok(None),
            PrimitiveType::ProceduralSamplerHandle => return Ok(None),
            PrimitiveType::ProceduralMaterialHandle => return Ok(None),
        }))
    }
}
