use std::{collections::HashMap, path::Path, time::Duration};

use ambient_ecs::EntityId;
use ambient_package as pkg;
use ambient_package_json as json;
use ambient_package_semantic as sema;
use ambient_shared_types::{
    primitive_component_definitions, ProceduralMaterialHandle, ProceduralMeshHandle,
    ProceduralSamplerHandle, ProceduralTextureHandle,
};

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use indexmap::IndexMap;
use sema::Item as _;

pub fn write(
    build_path: &Path,
    semantic: &sema::Semantic,
    package_item_id: sema::ItemId<sema::Package>,
) -> anyhow::Result<()> {
    let output_path = build_path.join("ambient_package.json");

    std::fs::write(
        output_path,
        serde_json::to_string(&json::Manifest {
            main_package_id: package_item_id.to_json(),
            items: semantic
                .items
                .iter()
                .map(|(k, v)| (k.0.to_string(), v.to_json()))
                .collect(),
        })?,
    )?;

    Ok(())
}

trait SemanticToJson {
    type Json;
    fn to_json(&self) -> Self::Json;
}

// Containers
impl<T: SemanticToJson> SemanticToJson for Option<T> {
    type Json = Option<T::Json>;
    fn to_json(&self) -> Self::Json {
        self.as_ref().map(|v| v.to_json())
    }
}

impl<T: SemanticToJson> SemanticToJson for Vec<T> {
    type Json = Vec<T::Json>;
    fn to_json(&self) -> Self::Json {
        self.iter().map(|v| v.to_json()).collect()
    }
}

impl<K: SemanticToJson, V: SemanticToJson> SemanticToJson for HashMap<K, V>
where
    <K as SemanticToJson>::Json: std::hash::Hash + Eq,
{
    type Json = HashMap<K::Json, V::Json>;
    fn to_json(&self) -> Self::Json {
        self.iter()
            .map(|(k, v)| (k.to_json(), v.to_json()))
            .collect()
    }
}

impl<K: SemanticToJson, V: SemanticToJson> SemanticToJson for IndexMap<K, V>
where
    <K as SemanticToJson>::Json: std::hash::Hash + Eq,
{
    type Json = IndexMap<K::Json, V::Json>;
    fn to_json(&self) -> Self::Json {
        self.iter()
            .map(|(k, v)| (k.to_json(), v.to_json()))
            .collect()
    }
}

// Base types
impl SemanticToJson for pkg::Identifier {
    type Json = json::Identifier;
    fn to_json(&self) -> Self::Json {
        self.to_string()
    }
}
impl SemanticToJson for pkg::SnakeCaseIdentifier {
    type Json = json::Identifier;
    fn to_json(&self) -> Self::Json {
        self.to_string()
    }
}
impl SemanticToJson for pkg::PascalCaseIdentifier {
    type Json = json::Identifier;
    fn to_json(&self) -> Self::Json {
        self.to_string()
    }
}

impl SemanticToJson for sema::ItemData {
    type Json = json::ItemData;
    fn to_json(&self) -> Self::Json {
        json::ItemData {
            id: self.id.to_json(),
            source: self.source.to_json(),
        }
    }
}

impl SemanticToJson for sema::ItemSource {
    type Json = json::ItemSource;
    fn to_json(&self) -> Self::Json {
        match self {
            sema::ItemSource::System => json::ItemSource::System,
            sema::ItemSource::Ambient => json::ItemSource::Ambient,
            sema::ItemSource::User => json::ItemSource::User,
        }
    }
}

impl<T: sema::Item + SemanticToJson> SemanticToJson for sema::ItemId<T>
where
    <T as SemanticToJson>::Json: ambient_package_json::Item,
{
    type Json = json::ItemId<T::Json>;
    fn to_json(&self) -> Self::Json {
        json::ItemId::from_u128(self.as_u128())
    }
}

impl<T: sema::Item + SemanticToJson> SemanticToJson for sema::ResolvableItemId<T>
where
    <T as SemanticToJson>::Json: ambient_package_json::Item,
{
    type Json = json::ItemId<T::Json>;
    fn to_json(&self) -> Self::Json {
        self.as_resolved().expect("unresolved item id").to_json()
    }
}

// Items
impl SemanticToJson for sema::ItemVariant {
    type Json = json::ItemVariant;
    fn to_json(&self) -> Self::Json {
        use json::ItemVariant as JIV;
        use sema::ItemVariant as SIV;

        match self {
            SIV::Component(v) => JIV::Component(v.to_json()),
            SIV::Concept(v) => JIV::Concept(v.to_json()),
            SIV::Message(v) => JIV::Message(v.to_json()),
            SIV::Type(v) => JIV::Type(v.to_json()),
            SIV::Attribute(v) => JIV::Attribute(v.to_json()),
            SIV::Scope(v) => JIV::Scope(v.to_json()),
            SIV::Package(v) => JIV::Package(v.to_json()),
        }
    }
}

impl SemanticToJson for sema::Component {
    type Json = json::Component;
    fn to_json(&self) -> Self::Json {
        json::Component {
            data: self.data().to_json(),
            name: self.name.to_json(),
            description: self.description.to_json(),
            type_: self.type_.to_json(),
            attributes: self.attributes.to_json(),
            default: self.default.to_json(),
        }
    }
}

impl SemanticToJson for sema::Concept {
    type Json = json::Concept;
    fn to_json(&self) -> Self::Json {
        json::Concept {
            data: self.data().to_json(),
            name: self.name.to_json(),
            description: self.description.to_json(),
            extends: self.extends.to_json(),
            required_components: self.required_components.to_json(),
            optional_components: self.optional_components.to_json(),
        }
    }
}
impl SemanticToJson for sema::ConceptValue {
    type Json = json::ConceptValue;
    fn to_json(&self) -> Self::Json {
        json::ConceptValue {
            description: self.description.to_json(),
            suggested: self.suggested.to_json(),
        }
    }
}

impl SemanticToJson for sema::Message {
    type Json = json::Message;
    fn to_json(&self) -> Self::Json {
        json::Message {
            data: self.data().to_json(),
            description: self.description.to_json(),
            fields: self.fields.to_json(),
        }
    }
}

impl SemanticToJson for sema::Type {
    type Json = json::Type;
    fn to_json(&self) -> Self::Json {
        json::Type {
            data: self.data().to_json(),
            inner: self.inner.to_json(),
        }
    }
}
impl SemanticToJson for sema::TypeInner {
    type Json = json::TypeInner;
    fn to_json(&self) -> Self::Json {
        match self {
            sema::TypeInner::Primitive(v) => json::TypeInner::Primitive(v.to_json()),
            sema::TypeInner::Vec(v) => json::TypeInner::Vec(v.to_json()),
            sema::TypeInner::Option(v) => json::TypeInner::Option(v.to_json()),
            sema::TypeInner::Enum(v) => json::TypeInner::Enum(v.to_json()),
        }
    }
}
macro_rules! impl_semantic_to_json_for_primitive_type {
    ($(($value:ident, $_:ty)),*) => {
        impl SemanticToJson for sema::PrimitiveType {
            type Json = json::PrimitiveType;
            fn to_json(&self) -> Self::Json {
                match self {
                    $(
                        sema::PrimitiveType::$value => json::PrimitiveType::$value,
                    )*
                }
            }
        }
    }
}
primitive_component_definitions!(impl_semantic_to_json_for_primitive_type);
impl SemanticToJson for sema::Enum {
    type Json = json::Enum;
    fn to_json(&self) -> Self::Json {
        json::Enum {
            description: self.description.to_json(),
            members: self.members.to_json(),
        }
    }
}

impl SemanticToJson for sema::Attribute {
    type Json = json::Attribute;
    fn to_json(&self) -> Self::Json {
        json::Attribute {
            data: self.data().to_json(),
        }
    }
}

impl SemanticToJson for sema::Scope {
    type Json = json::Scope;
    fn to_json(&self) -> Self::Json {
        json::Scope {
            data: self.data.to_json(),
            imports: self.imports.to_json(),
            scopes: self.scopes.to_json(),
            components: self.components.to_json(),
            concepts: self.concepts.to_json(),
            messages: self.messages.to_json(),
            types: self.types.to_json(),
            attributes: self.attributes.to_json(),
        }
    }
}

impl SemanticToJson for sema::Package {
    type Json = json::Package;
    fn to_json(&self) -> Self::Json {
        json::Package {
            data: self.data.to_json(),
            dependencies: self
                .dependencies
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_json()))
                .collect(),
        }
    }
}
impl SemanticToJson for sema::Dependency {
    type Json = json::Dependency;
    fn to_json(&self) -> Self::Json {
        json::Dependency {
            id: self.id.to_json(),
            enabled: self.enabled,
        }
    }
}

// Values
impl SemanticToJson for sema::Value {
    type Json = json::Value;
    fn to_json(&self) -> Self::Json {
        match self {
            sema::Value::Scalar(v) => json::Value::Scalar(v.to_json()),
            sema::Value::Vec(v) => json::Value::Vec(v.to_json()),
            sema::Value::Option(v) => json::Value::Option(v.to_json()),
            sema::Value::Enum(ty, id) => json::Value::Enum(json::EnumValue {
                ty: ty.to_json(),
                member: id.to_json(),
            }),
        }
    }
}
impl SemanticToJson for sema::ResolvableValue {
    type Json = json::Value;
    fn to_json(&self) -> Self::Json {
        self.as_resolved().expect("unresolved value").to_json()
    }
}
impl SemanticToJson for sema::ScalarValue {
    type Json = json::ScalarValue;
    fn to_json(&self) -> Self::Json {
        match self {
            sema::ScalarValue::Empty(v) => json::ScalarValue::Empty(v.to_json()),
            sema::ScalarValue::Bool(v) => json::ScalarValue::Bool(v.to_json()),
            sema::ScalarValue::EntityId(v) => json::ScalarValue::EntityId(v.to_json()),
            sema::ScalarValue::F32(v) => json::ScalarValue::F32(v.to_json()),
            sema::ScalarValue::F64(v) => json::ScalarValue::F64(v.to_json()),
            sema::ScalarValue::Mat4(v) => json::ScalarValue::Mat4(v.to_json()),
            sema::ScalarValue::Quat(v) => json::ScalarValue::Quat(v.to_json()),
            sema::ScalarValue::String(v) => json::ScalarValue::String(v.to_json()),
            sema::ScalarValue::U8(v) => json::ScalarValue::U8(v.to_json()),
            sema::ScalarValue::U16(v) => json::ScalarValue::U16(v.to_json()),
            sema::ScalarValue::U32(v) => json::ScalarValue::U32(v.to_json()),
            sema::ScalarValue::U64(v) => json::ScalarValue::U64(v.to_json()),
            sema::ScalarValue::I8(v) => json::ScalarValue::I8(v.to_json()),
            sema::ScalarValue::I16(v) => json::ScalarValue::I16(v.to_json()),
            sema::ScalarValue::I32(v) => json::ScalarValue::I32(v.to_json()),
            sema::ScalarValue::I64(v) => json::ScalarValue::I64(v.to_json()),
            sema::ScalarValue::Vec2(v) => json::ScalarValue::Vec2(v.to_json()),
            sema::ScalarValue::Vec3(v) => json::ScalarValue::Vec3(v.to_json()),
            sema::ScalarValue::Vec4(v) => json::ScalarValue::Vec4(v.to_json()),
            sema::ScalarValue::Uvec2(v) => json::ScalarValue::Uvec2(v.to_json()),
            sema::ScalarValue::Uvec3(v) => json::ScalarValue::Uvec3(v.to_json()),
            sema::ScalarValue::Uvec4(v) => json::ScalarValue::Uvec4(v.to_json()),
            sema::ScalarValue::Ivec2(v) => json::ScalarValue::Ivec2(v.to_json()),
            sema::ScalarValue::Ivec3(v) => json::ScalarValue::Ivec3(v.to_json()),
            sema::ScalarValue::Ivec4(v) => json::ScalarValue::Ivec4(v.to_json()),
            sema::ScalarValue::Duration(v) => json::ScalarValue::Duration(v.to_json()),
            sema::ScalarValue::ProceduralMeshHandle(v) => {
                json::ScalarValue::ProceduralMeshHandle(v.to_json())
            }
            sema::ScalarValue::ProceduralTextureHandle(v) => {
                json::ScalarValue::ProceduralTextureHandle(v.to_json())
            }
            sema::ScalarValue::ProceduralSamplerHandle(v) => {
                json::ScalarValue::ProceduralSamplerHandle(v.to_json())
            }
            sema::ScalarValue::ProceduralMaterialHandle(v) => {
                json::ScalarValue::ProceduralMaterialHandle(v.to_json())
            }
        }
    }
}
macro_rules! identity_to_json {
    ($($ty:ty),*) => {
        $(
            impl SemanticToJson for $ty {
                type Json = $ty;
                fn to_json(&self) -> Self::Json {
                    self.clone()
                }
            }
        )*
    };
}
identity_to_json![
    (),
    bool,
    f32,
    f64,
    String,
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    Duration
];

impl SemanticToJson for u128 {
    type Json = String;
    fn to_json(&self) -> Self::Json {
        self.to_string()
    }
}

impl SemanticToJson for EntityId {
    type Json = json::EntityId;
    fn to_json(&self) -> Self::Json {
        self.to_base64()
    }
}

impl SemanticToJson for Mat4 {
    type Json = json::Mat4;
    fn to_json(&self) -> Self::Json {
        self.to_cols_array()
    }
}
macro_rules! impl_semantic_to_json_for_array_types {
    ($($ty:ident),*) => {
        $(
            impl SemanticToJson for $ty {
                type Json = json :: $ty;
                fn to_json(&self) -> Self::Json {
                    self.to_array()
                }
            }
        )*
    };
}
impl_semantic_to_json_for_array_types![
    Quat, Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, IVec2, IVec3, IVec4
];

macro_rules! impl_semantic_to_json_for_procedural_handles {
    ($($ty:ident),*) => {
        $(
            impl SemanticToJson for $ty {
                type Json = json::$ty;
                fn to_json(&self) -> Self::Json {
                    self.as_u128().to_json()
                }
            }
        )*
    };
}
impl_semantic_to_json_for_procedural_handles![
    ProceduralMeshHandle,
    ProceduralTextureHandle,
    ProceduralSamplerHandle,
    ProceduralMaterialHandle
];
