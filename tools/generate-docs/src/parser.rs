use rustdoc_types::{
    Crate, Enum as RdEnum, GenericArg, GenericArgs, Id, Item, ItemEnum, Path as RdPath, Struct as RdStruct, StructKind, Type as RdType,
    VariantKind,
};

use crate::helpers::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub docs: String,
    pub name: String,
    pub ty: Type,
    pub default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Primitive(String),
    Struct(Struct),
    Enum(Enum),
    List(Box<Type>),
    // Dictionary(Box<Type>, Box<Type>),
    Option(Box<Type>),
    StringLiteral(String),

    String,
    AssetUrl,
    Vec2,
    Vec3,
    Vec4,
    EntityData,
}
impl Type {
    pub fn convert_item(krate: &Crate, item: &Item) -> Type {
        match &item.inner {
            ItemEnum::Struct(s) => Type::Struct(Struct { name: item.name.clone().unwrap(), fields: convert_struct_fields(krate, s) }),
            ItemEnum::Enum(e) => Type::Enum(Enum::convert(krate, item.name.clone().unwrap(), &item.attrs, e)),
            _ => unimplemented!("{:?}", item),
        }
    }

    fn convert_type(krate: &Crate, value: &RdType) -> Type {
        match value {
            RdType::ResolvedPath(p) => {
                if p.name == "Vec" {
                    Type::List(Box::new(Self::get_first_type_from_generic(krate, p)))
                } else if p.name == "Option" {
                    Type::Option(Box::new(Self::get_first_type_from_generic(krate, p)))
                } else if p.name == "String" {
                    Type::String
                } else if p.name == "AssetUrl" {
                    Type::AssetUrl
                } else if p.name == "Vec2" {
                    Type::Vec2
                } else if p.name == "Vec3" {
                    Type::Vec3
                } else if p.name == "Vec4" {
                    Type::Vec4
                } else if p.name == "EntityData" {
                    Type::EntityData
                } else if p.name == "Box" {
                    Self::get_first_type_from_generic(krate, p)
                } else {
                    let (krate, item) = p.get(krate);
                    Self::convert_item(krate, item)
                }
            }
            // RdType::DynTrait(_) => todo!(),
            // RdType::Generic(_) => todo!(),
            RdType::Primitive(p) => Type::Primitive(p.clone()),
            // RdType::FunctionPointer(_) => todo!(),
            // RdType::Tuple(_) => todo!(),
            // RdType::Slice(_) => todo!(),
            // RdType::Array { type_, len } => todo!(),
            // RdType::ImplTrait(_) => todo!(),
            // RdType::Infer => todo!(),
            // RdType::BorrowedRef { lifetime, mutable, type_ } => todo!(),
            // RdType::QualifiedPath { name, args, self_type, trait_ } => todo!(),
            _ => unimplemented!("unsupported type {value:?}"),
        }
    }

    fn get_first_type_from_generic(krate: &Crate, p: &RdPath) -> Type {
        match p.args.as_ref().unwrap().as_ref() {
            GenericArgs::AngleBracketed { args, .. } => match &args[0] {
                GenericArg::Lifetime(_) => todo!(),
                GenericArg::Type(t) => Self::convert_type(krate, t),
                GenericArg::Const(_) => todo!(),
                GenericArg::Infer => todo!(),
            },
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {
    pub name: String,
    pub tag: Option<String>,
    pub variants: Vec<(String, Option<Type>)>,
}
impl Enum {
    fn convert(krate: &Crate, name: String, attrs: &[String], value: &RdEnum) -> Self {
        let make_variant = |id: &Id| {
            let item = id.get(krate);
            let variant = item.inner.to_variant().unwrap();
            let ty = match &variant.kind {
                VariantKind::Plain => None,
                VariantKind::Tuple(t) => {
                    assert_eq!(t.len(), 1);
                    // Get the first non-hidden field
                    let item = t.iter().find_map(|o| o.as_ref()).map(|id| id.get(krate)).unwrap();
                    // Get its inner type from the pub struct field
                    let ty = item.inner.to_struct_field_type().unwrap();
                    Some(Type::convert_type(krate, ty))
                }
                VariantKind::Struct { fields, .. } => {
                    Some(Type::Struct(Struct { name: name.clone(), fields: convert_plain_fields(krate, &fields) }))
                }
            };

            (item.name.clone().unwrap(), ty)
        };

        // Extremely not robust, even more so than the rest of this code. Get the bare minimum working first.
        let tag = attrs.iter().find(|a| a.contains("tag = \"")).and_then(|a| {
            let (_, rhs) = a.split_once('"')?;
            let (lhs, _) = rhs.split_once('"')?;
            Some(lhs.to_string())
        });

        Self { name: name.clone(), tag, variants: value.variants.iter().map(make_variant).collect() }
    }
}

fn convert_struct_fields(krate: &Crate, strukt: &RdStruct) -> Vec<Field> {
    match &strukt.kind {
        StructKind::Unit => todo!(),
        StructKind::Tuple(_) => todo!(),
        StructKind::Plain { fields, .. } => convert_plain_fields(krate, fields),
    }
}

fn convert_plain_fields(krate: &Crate, fields: &[Id]) -> Vec<Field> {
    fields
        .iter()
        .flat_map(|i| {
            let field = i.get(krate);
            let ty = field.inner.to_struct_field_type()?;
            let docs = field.docs.clone().unwrap_or_else(|| "Undocumented!".to_string());
            let default = field.attrs.iter().any(|a| a.contains("serde(default"));

            Some(Field { docs, name: field.name.clone().unwrap(), ty: Type::convert_type(krate, ty), default })
        })
        .collect()
}
