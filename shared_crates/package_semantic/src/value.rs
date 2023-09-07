use std::{
    f64::consts::{FRAC_PI_2, PI},
    fmt,
    time::Duration,
};

use super::{PrimitiveType, Type};
use crate::{ItemId, ItemMap, TypeInner};

use ambient_package::PascalCaseIdentifier;
use ambient_shared_types::{
    primitive_component_definitions, ProceduralMaterialHandle, ProceduralMeshHandle,
    ProceduralSamplerHandle, ProceduralTextureHandle,
};
use anyhow::Context as AnyhowContext;
use glam::{IVec2, IVec3, IVec4, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

pub type EntityId = u128;

#[derive(Clone, PartialEq, Debug)]
pub enum ResolvableValue {
    Unresolved(toml::Value),
    Resolved(Value),
}
impl ResolvableValue {
    pub(crate) fn resolve_in_place(
        &mut self,
        items: &ItemMap,
        id: ItemId<Type>,
    ) -> anyhow::Result<()> {
        if let Self::Unresolved(value) = self {
            *self = Self::Resolved(Value::from_toml(value, items, id)?);
        }
        Ok(())
    }

    pub fn as_resolved(&self) -> Option<&Value> {
        match self {
            Self::Resolved(value) => Some(value),
            _ => None,
        }
    }
}

macro_rules! define_scalar_value {
    ($(($value:ident, $type:ty)),*) => {
        paste::paste! {
            #[derive(Debug, Clone, PartialEq)]
            pub enum ScalarValue {
                $(
                    $value($type),
                )*
            }
            $(
                impl From<$type> for ScalarValue {
                    fn from(value: $type) -> Self {
                        Self::$value(value)
                    }
                }
            )*
            impl fmt::Display for ScalarValue {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    match self {
                        $(
                            Self::$value(value) => fmt::Debug::fmt(value, f),
                        )*
                    }
                }
            }
        }
    };
}
primitive_component_definitions!(define_scalar_value);

impl ScalarValue {
    pub fn from_toml(value: &toml::Value, ty: PrimitiveType) -> anyhow::Result<Self> {
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
            Ok(match v {
                toml::Value::String(s) if s.as_str() == "PI" => PI,
                toml::Value::String(s) if s.as_str() == "-PI" => -PI,
                toml::Value::String(s) if s.as_str() == "-FRAC_PI_2" => -FRAC_PI_2,
                toml::Value::String(s) if s.as_str() == "FRAC_PI_2" => FRAC_PI_2,
                _ => v
                    .as_float()
                    .with_context(|| format!("Expected float, got {:?}", v))?,
            })
        }

        let v = value;
        Ok(match ty {
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
            PrimitiveType::Mat4 => Self::Mat4({
                match v {
                    toml::Value::String(s) if s.as_str() == "Identity" => Mat4::IDENTITY,
                    _ => Mat4::from_cols_array(&as_array(v, |v| Ok(as_float(v)? as f32))?),
                }
            }),
            PrimitiveType::Quat => Self::Quat(match v {
                toml::Value::String(s) if s.as_str() == "Identity" => Quat::IDENTITY,
                _ => Quat::from_array(as_array(v, |v| Ok(as_float(v)? as f32))?),
            }),
            PrimitiveType::String => Self::String(as_str(v)?.to_string()),
            PrimitiveType::U8 => Self::U8(as_integer(v)? as u8),
            PrimitiveType::U16 => Self::U16(as_integer(v)? as u16),
            PrimitiveType::U32 => Self::U32(as_integer(v)? as u32),
            PrimitiveType::U64 => Self::U64(as_integer(v)? as u64),
            PrimitiveType::I8 => Self::I8(as_integer(v)? as i8),
            PrimitiveType::I16 => Self::I16(as_integer(v)? as i16),
            PrimitiveType::I32 => Self::I32(as_integer(v)? as i32),
            PrimitiveType::I64 => Self::I64(as_integer(v)?),
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
            PrimitiveType::Ivec2 => Self::Ivec2(IVec2::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as i32)
            })?)),
            PrimitiveType::Ivec3 => Self::Ivec3(IVec3::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as i32)
            })?)),
            PrimitiveType::Ivec4 => Self::Ivec4(IVec4::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as i32)
            })?)),
            PrimitiveType::Duration => anyhow::bail!("unsupported value to load from TOML"),
            PrimitiveType::ProceduralMeshHandle => {
                anyhow::bail!("unsupported value to load from TOML")
            }
            PrimitiveType::ProceduralTextureHandle => {
                anyhow::bail!("unsupported value to load from TOML")
            }
            PrimitiveType::ProceduralSamplerHandle => {
                anyhow::bail!("unsupported value to load from TOML")
            }
            PrimitiveType::ProceduralMaterialHandle => {
                anyhow::bail!("unsupported value to load from TOML")
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Scalar(ScalarValue),
    Vec(Vec<ScalarValue>),
    Option(Option<ScalarValue>),
    Enum(ItemId<Type>, PascalCaseIdentifier),
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(v) => fmt::Display::fmt(v, f),
            Self::Vec(v) => {
                write!(f, "[")?;
                for (i, v) in v.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    fmt::Display::fmt(v, f)?;
                }
                write!(f, "]")
            }
            Self::Option(v) => {
                if let Some(v) = v {
                    write!(f, "Some({})", v)
                } else {
                    write!(f, "None")
                }
            }
            Self::Enum(ty, v) => write!(f, "{ty}::{v}"),
        }
    }
}
impl Value {
    pub(crate) fn from_toml(
        value: &toml::Value,
        items: &ItemMap,
        ty_id: ItemId<Type>,
    ) -> anyhow::Result<Self> {
        let ty = &*items.get(ty_id);
        Ok(match &ty.inner {
            TypeInner::Primitive(pt) => Self::Scalar(ScalarValue::from_toml(value, *pt)?),
            TypeInner::Vec(v) => {
                let inner_ty = &*items.get(*v);
                let inner_ty = inner_ty.inner.as_primitive().with_context(|| {
                    format!("Expected primitive type, got {:?}", inner_ty.inner)
                })?;

                let arr = value
                    .as_array()
                    .with_context(|| format!("Expected array, got {:?}", value))?;

                Self::Vec(
                    arr.iter()
                        .map(|v| ScalarValue::from_toml(v, inner_ty))
                        .collect::<anyhow::Result<_>>()?,
                )
            }
            TypeInner::Option(o) => {
                let inner_ty = &*items.get(*o);
                let inner_ty = inner_ty.inner.as_primitive().with_context(|| {
                    format!("Expected primitive type, got {:?}", inner_ty.inner)
                })?;

                let arr = value
                    .as_array()
                    .with_context(|| format!("Expected array, got {:?}", value))?;
                if arr.len() > 1 {
                    anyhow::bail!("Expected array of length 0 or 1, got {:?}", value);
                }

                if arr.is_empty() {
                    Self::Option(None)
                } else {
                    Self::Option(Some(ScalarValue::from_toml(&arr[0], inner_ty)?))
                }
            }
            TypeInner::Enum(e) => {
                let variant = value.as_str().with_context(|| {
                    format!("Expected string for enum variant, got {:?}", value)
                })?;

                let variant = e
                    .members
                    .iter()
                    .find(|(name, _description)| name.as_str() == variant)
                    .with_context(|| {
                        format!(
                            "Expected enum variant to be one of {:?}, got {:?}",
                            e.members, variant
                        )
                    })?;

                Self::Enum(ty_id, variant.0.clone())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{create_root_scope, Enum, ItemData, ItemSource};

    use super::*;

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_scalar_value_from_toml() -> anyhow::Result<()> {
        {
            let value = toml::Value::Array(vec![]);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::Empty)?;
            assert_eq!(scalar_value, ScalarValue::Empty(()));
        }

        {
            let value = toml::Value::from(true);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::Bool)?;
            assert_eq!(scalar_value, ScalarValue::Bool(true));
        }

        {
            let value = toml::Value::String("MTIzNDU2NzhBQkNERUZHSA==".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::EntityId)?;
            assert_eq!(
                scalar_value,
                ScalarValue::EntityId(EntityId::from_le_bytes(*b"12345678ABCDEFGH"))
            )
        };

        {
            let value = toml::Value::from(3.14);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F32)?;
            assert_eq!(scalar_value, ScalarValue::F32(3.14));

            let value = toml::Value::from("PI".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F32)?;
            assert_eq!(scalar_value, ScalarValue::F32(PI as f32));

            let value = toml::Value::from("FRAC_PI_2".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F32)?;
            assert_eq!(scalar_value, ScalarValue::F32(FRAC_PI_2 as f32));

            let value = toml::Value::from("-PI".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F32)?;
            assert_eq!(scalar_value, ScalarValue::F32(-PI as f32));

            let value = toml::Value::from("-FRAC_PI_2".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F32)?;
            assert_eq!(scalar_value, ScalarValue::F32(-FRAC_PI_2 as f32));
        }

        {
            let value = toml::Value::from(3.14);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F64)?;
            assert_eq!(scalar_value, ScalarValue::F64(3.14));

            let value = toml::Value::from("PI".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F64)?;
            assert_eq!(scalar_value, ScalarValue::F64(PI));

            let value = toml::Value::from("FRAC_PI_2".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F64)?;
            assert_eq!(scalar_value, ScalarValue::F64(FRAC_PI_2));

            let value = toml::Value::from("-PI".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F64)?;
            assert_eq!(scalar_value, ScalarValue::F64(-PI));

            let value = toml::Value::from("-FRAC_PI_2".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::F64)?;
            assert_eq!(scalar_value, ScalarValue::F64(-FRAC_PI_2));
        }

        {
            const MAT4_TEST: [f32; 16] = [
                1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0,
            ];
            let value = toml::Value::from(MAT4_TEST.to_vec());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::Mat4)?;
            assert_eq!(
                scalar_value,
                ScalarValue::Mat4(glam::Mat4::from_cols_array(&MAT4_TEST))
            );

            let value = toml::Value::from("Identity".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::Mat4)?;
            assert_eq!(scalar_value, ScalarValue::Mat4(glam::Mat4::IDENTITY));
        };

        {
            const QUAT_TEST: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
            let value = toml::Value::from(QUAT_TEST.to_vec());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::Quat)?;
            assert_eq!(
                scalar_value,
                ScalarValue::Quat(glam::Quat::from_array(QUAT_TEST))
            );

            let value = toml::Value::from("Identity".to_string());
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::Quat)?;
            assert_eq!(scalar_value, ScalarValue::Quat(glam::Quat::IDENTITY));
        };

        {
            let value = toml::Value::from("hello");
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::String)?;
            assert_eq!(scalar_value, ScalarValue::String("hello".to_string()));
        }

        {
            let value = toml::Value::from(42);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::U8)?;
            assert_eq!(scalar_value, ScalarValue::U8(42));
        }

        {
            let value = toml::Value::from(42);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::U16)?;
            assert_eq!(scalar_value, ScalarValue::U16(42));
        }

        {
            let value = toml::Value::from(42);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::U32)?;
            assert_eq!(scalar_value, ScalarValue::U32(42));
        }

        {
            let value = toml::Value::from(42);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::U64)?;
            assert_eq!(scalar_value, ScalarValue::U64(42));
        }

        {
            let value = toml::Value::from(-42);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::I8)?;
            assert_eq!(scalar_value, ScalarValue::I8(-42));
        }

        {
            let value = toml::Value::from(-42);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::I16)?;
            assert_eq!(scalar_value, ScalarValue::I16(-42));
        }

        {
            let value = toml::Value::from(-42);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::I32)?;
            assert_eq!(scalar_value, ScalarValue::I32(-42));
        }

        {
            let value = toml::Value::from(-42);
            let scalar_value = ScalarValue::from_toml(&value, PrimitiveType::I64)?;
            assert_eq!(scalar_value, ScalarValue::I64(-42));
        }

        Ok(())
    }

    #[test]
    fn test_value_from_toml() {
        let mut items = ItemMap::default();
        let (root_scope, _) = create_root_scope(&mut items).unwrap();

        let enum_type = items.add(Type::new(
            ItemData {
                parent_id: None,
                id: PascalCaseIdentifier::new("MyEnum").unwrap().into(),
                source: ItemSource::User,
            },
            TypeInner::Enum(Enum {
                description: None,
                members: ["A", "B", "C"]
                    .into_iter()
                    .map(|s| (PascalCaseIdentifier::new(s).unwrap(), "".to_string()))
                    .collect(),
            }),
        ));

        let root_scope = items.get(root_scope);

        let bool_type = *root_scope
            .types
            .get(&PascalCaseIdentifier::new("Bool").unwrap())
            .unwrap();

        let u32_type = *root_scope
            .types
            .get(&PascalCaseIdentifier::new("U32").unwrap())
            .unwrap();
        let vec_u32_type = items.get_vec_id(u32_type);

        let string_type = *root_scope
            .types
            .get(&PascalCaseIdentifier::new("String").unwrap())
            .unwrap();
        let opt_string_type = items.get_option_id(string_type);

        assert_eq!(
            Value::from_toml(&toml::Value::Boolean(true), &items, bool_type).unwrap(),
            Value::Scalar(ScalarValue::Bool(true))
        );

        assert_eq!(
            Value::from_toml(
                &toml::Value::Array(vec![
                    toml::Value::Integer(1),
                    toml::Value::Integer(2),
                    toml::Value::Integer(3)
                ]),
                &items,
                vec_u32_type
            )
            .unwrap(),
            Value::Vec(vec![
                ScalarValue::U32(1),
                ScalarValue::U32(2),
                ScalarValue::U32(3)
            ])
        );

        assert_eq!(
            Value::from_toml(
                &toml::Value::Array(vec![toml::Value::String("hello".to_string())]),
                &items,
                opt_string_type
            )
            .unwrap(),
            Value::Option(Some(ScalarValue::String("hello".to_string())))
        );

        assert_eq!(
            Value::from_toml(&toml::Value::Array(vec![]), &items, opt_string_type).unwrap(),
            Value::Option(None)
        );

        assert_eq!(
            Value::from_toml(&toml::Value::String("B".to_string()), &items, enum_type).unwrap(),
            Value::Enum(enum_type, PascalCaseIdentifier::new("B").unwrap()),
        );
    }
}
