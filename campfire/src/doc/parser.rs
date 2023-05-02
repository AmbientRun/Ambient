use std::fmt::Display;

use rustdoc_types as rdt;

use super::{context::Context, helpers::ItemHelpers};

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
    Entity,
}
impl Type {
    pub fn convert_item(ctx: &Context, source_crate: &rdt::Crate, item: &rdt::Item) -> Type {
        match &item.inner {
            rdt::ItemEnum::Struct(s) => Type::Struct(Struct {
                name: item.name.clone().unwrap(),
                fields: convert_struct_fields(ctx, source_crate, s),
            }),
            rdt::ItemEnum::Enum(e) => Type::Enum(Enum::convert(
                ctx,
                source_crate,
                item.name.clone().unwrap(),
                &item.attrs,
                e,
            )),
            _ => unimplemented!("{:?}", item),
        }
    }

    fn convert_type(ctx: &Context, source_crate: &rdt::Crate, value: &rdt::Type) -> Type {
        match value {
            rdt::Type::ResolvedPath(p) => {
                if p.name == "Vec" {
                    Type::List(Box::new(Self::get_first_type_from_generic(
                        ctx,
                        source_crate,
                        p,
                    )))
                } else if p.name == "Option" {
                    Type::Option(Box::new(Self::get_first_type_from_generic(
                        ctx,
                        source_crate,
                        p,
                    )))
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
                } else if p.name == "Entity" {
                    Type::Entity
                } else if p.name == "Box" {
                    Self::get_first_type_from_generic(ctx, source_crate, p)
                } else {
                    let (krate, item) = ctx.get_by_path(source_crate, p).expect("no item found");
                    Self::convert_item(ctx, krate, item)
                }
            }
            // rdt::Type::DynTrait(_) => todo!(),
            // rdt::Type::Generic(_) => todo!(),
            rdt::Type::Primitive(p) => Type::Primitive(p.clone()),
            // rdt::Type::FunctionPointer(_) => todo!(),
            // rdt::Type::Tuple(_) => todo!(),
            // rdt::Type::Slice(_) => todo!(),
            // rdt::Type::Array { type_, len } => todo!(),
            // rdt::Type::ImplTrait(_) => todo!(),
            // rdt::Type::Infer => todo!(),
            // rdt::Type::BorrowedRef { lifetime, mutable, type_ } => todo!(),
            // rdt::Type::QualifiedPath { name, args, self_type, trait_ } => todo!(),
            _ => unimplemented!("unsupported type {value:?}"),
        }
    }

    fn get_first_type_from_generic(
        ctx: &Context,
        source_crate: &rdt::Crate,
        path: &rdt::Path,
    ) -> Type {
        match path.args.as_ref().unwrap().as_ref() {
            rdt::GenericArgs::AngleBracketed { args, .. } => match &args[0] {
                rdt::GenericArg::Lifetime(_) => todo!(),
                rdt::GenericArg::Type(t) => Self::convert_type(ctx, source_crate, t),
                rdt::GenericArg::Const(_) => todo!(),
                rdt::GenericArg::Infer => todo!(),
            },
            _ => unimplemented!(),
        }
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{}", p),
            Type::Struct(s) => write!(f, "{}", s.name),
            Type::Enum(e) => write!(f, "{}", e.name),
            Type::List(t) => write!(f, "Vec<{}>", t),
            Type::Option(t) => write!(f, "Option<{}>", t),
            Type::StringLiteral(s) => write!(f, "{}", s),
            Type::String => write!(f, "String"),
            Type::AssetUrl => write!(f, "AssetUrl"),
            Type::Vec2 => write!(f, "Vec2"),
            Type::Vec3 => write!(f, "Vec3"),
            Type::Vec4 => write!(f, "Vec4"),
            Type::Entity => write!(f, "Entity"),
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
    pub variants: Vec<(String, String, Option<Type>)>,
}
impl Enum {
    fn convert(
        ctx: &Context,
        source_crate: &rdt::Crate,
        name: String,
        attrs: &[String],
        value: &rdt::Enum,
    ) -> Self {
        let make_variant = |id: &rdt::Id| {
            let item = source_crate.index.get(id).expect("invalid id");
            let variant = item.inner.to_variant().unwrap();
            let ty = match &variant.kind {
                rdt::VariantKind::Plain => None,
                rdt::VariantKind::Tuple(t) => {
                    assert_eq!(t.len(), 1);
                    // Get the first non-hidden field
                    let item = t
                        .iter()
                        .find_map(|o| o.as_ref())
                        .map(|id| source_crate.index.get(id).expect("invalid id"))
                        .unwrap();
                    // Get its inner type from the pub struct field
                    let ty = item.inner.to_struct_field_type().unwrap();
                    Some(Type::convert_type(ctx, source_crate, ty))
                }
                rdt::VariantKind::Struct { fields, .. } => Some(Type::Struct(Struct {
                    name: name.clone(),
                    fields: convert_plain_fields(ctx, source_crate, fields),
                })),
            };

            (
                item.name.clone().unwrap(),
                item.docs.clone().unwrap_or_default(),
                ty,
            )
        };

        // Extremely not robust, even more so than the rest of this code. Get the bare minimum working first.
        let tag = attrs.iter().find(|a| a.contains("tag = \"")).and_then(|a| {
            let (_, rhs) = a.split_once('"')?;
            let (lhs, _) = rhs.split_once('"')?;
            Some(lhs.to_string())
        });

        Self {
            name: name.clone(),
            tag,
            variants: value.variants.iter().map(make_variant).collect(),
        }
    }
}

fn convert_struct_fields(ctx: &Context, krate: &rdt::Crate, strukt: &rdt::Struct) -> Vec<Field> {
    match &strukt.kind {
        rdt::StructKind::Unit => todo!(),
        rdt::StructKind::Tuple(_) => todo!(),
        rdt::StructKind::Plain { fields, .. } => convert_plain_fields(ctx, krate, fields),
    }
}

fn convert_plain_fields(ctx: &Context, krate: &rdt::Crate, fields: &[rdt::Id]) -> Vec<Field> {
    fields
        .iter()
        .flat_map(|id| {
            let field = krate.index.get(id).expect("invalid id");
            let ty = field.inner.to_struct_field_type()?;
            let docs = field
                .docs
                .clone()
                .unwrap_or_else(|| "Undocumented!".to_string());
            let default = field.attrs.iter().any(|a| a.contains("serde(default"));

            Some(Field {
                docs,
                name: field.name.clone().unwrap(),
                ty: Type::convert_type(ctx, krate, ty),
                default,
            })
        })
        .collect()
}
