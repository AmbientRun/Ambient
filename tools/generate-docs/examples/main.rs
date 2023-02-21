//! This code is extremely ugly. Its sole purpose is to convert our serde types
//! into something that can be used in documentation.
use std::{collections::HashMap, fs::File};

use anyhow::Context;
use helpers::*;
use once_cell::sync::Lazy;
use rustdoc_types::{Crate, Id};
use std::io::Write;

static CRATES: Lazy<HashMap<String, Crate>> = Lazy::new(|| {
    ["crates/physics/Cargo.toml", "crates/model_import/Cargo.toml", "crates/build/Cargo.toml"]
        .iter()
        .map(|n| {
            let build = rustdoc_json::Builder::default()
                .toolchain("nightly")
                .document_private_items(true)
                .manifest_path(n)
                .silent(true)
                .build()
                .unwrap();

            let krate = serde_json::from_str(&std::fs::read_to_string(build).unwrap()).unwrap();
            (n.to_string(), krate)
        })
        .collect()
});
static PATH_TO_CRATE_AND_ID: Lazy<HashMap<String, (String, Id)>> = Lazy::new(|| {
    CRATES
        .iter()
        .flat_map(|(n, krate)| krate.paths.iter().filter(|p| p.1.crate_id == 0).map(|p| (p.1.path.join("::"), (n.clone(), p.0.clone()))))
        .collect()
});

fn main() -> anyhow::Result<()> {
    Lazy::force(&PATH_TO_CRATE_AND_ID);

    generate_pipeline()?;

    Ok(())
}

fn generate_pipeline() -> anyhow::Result<()> {
    let (crate_path, id) = PATH_TO_CRATE_AND_ID.get("ambient_build::pipelines::Pipeline").context("no pipeline struct found")?;
    let build_crate = CRATES.get(crate_path).unwrap();
    let pipeline = id.get(build_crate);
    let ty = parser::Type::convert_item(build_crate, pipeline);

    let mut file = std::fs::File::create("docs/src/pipeline.d.ts")?;
    writeln!(file, "// pipeline.json")?;

    for (alias, ts_type) in [
        ("u32", "number"),
        ("f32", "number"),
        ("Vec2", "[number, number]"),
        ("Vec3", "[number, number, number]"),
        ("Vec4", "[number, number, number, number]"),
        ("EntityData", "{[component_id: string]: any}"),
        ("AssetUrl", "string"),
    ] {
        writeln!(file, "export type {alias} = {ts_type};")?;
    }

    writeln!(file)?;

    write!(file, "export type Pipeline = ")?;
    ty.generate_dts(&mut file, 1)?;

    Ok(())
}

trait GenerateDTS {
    fn generate_dts(&self, file: &mut File, indent: usize) -> anyhow::Result<()>;
    fn generate_dts_unwrapped(&self, file: &mut File, indent: usize) -> anyhow::Result<()>;
}
impl GenerateDTS for parser::Type {
    fn generate_dts(&self, file: &mut File, indent: usize) -> anyhow::Result<()> {
        match self {
            parser::Type::Struct(s) => s.generate_dts(file, indent)?,
            parser::Type::Enum(s) => s.generate_dts(file, indent)?,
            parser::Type::List(s) => {
                let is_enum = matches!(**s, parser::Type::Enum(_));
                if is_enum {
                    write!(file, "(")?;
                }
                s.generate_dts(file, indent)?;
                if is_enum {
                    write!(file, ")")?;
                }
                write!(file, "[]")?
            }
            parser::Type::Option(s) => {
                s.generate_dts(file, indent)?;
                write!(file, " | null")?
            }
            _ => self.generate_dts_unwrapped(file, indent)?,
        }
        Ok(())
    }
    fn generate_dts_unwrapped(&self, file: &mut File, indent: usize) -> anyhow::Result<()> {
        match self {
            parser::Type::Struct(s) => s.generate_dts_unwrapped(file, indent)?,
            parser::Type::Enum(s) => s.generate_dts_unwrapped(file, indent)?,
            parser::Type::List(s) => {
                let is_enum = matches!(**s, parser::Type::Enum(_));
                if is_enum {
                    write!(file, "(")?;
                }
                s.generate_dts_unwrapped(file, indent)?;
                if is_enum {
                    write!(file, ")")?;
                }
                write!(file, "[]")?
            }
            parser::Type::Option(s) => {
                s.generate_dts_unwrapped(file, indent)?;
                write!(file, " | null")?
            }
            parser::Type::Primitive(s) => write!(
                file,
                "{}",
                match s.as_str() {
                    "bool" => "boolean",
                    other => other,
                }
            )?,
            parser::Type::StringLiteral(s) => write!(file, "{s:?}")?,
            parser::Type::String => write!(file, "string")?,
            parser::Type::AssetUrl => write!(file, "AssetUrl")?,
            parser::Type::Vec2 => write!(file, "Vec2")?,
            parser::Type::Vec3 => write!(file, "Vec3")?,
            parser::Type::Vec4 => write!(file, "Vec4")?,
            parser::Type::EntityData => write!(file, "EntityData")?,
        }
        Ok(())
    }
}
impl GenerateDTS for parser::Struct {
    fn generate_dts(&self, file: &mut File, indent: usize) -> anyhow::Result<()> {
        writeln!(file, "{{")?;
        self.generate_dts_unwrapped(file, indent)?;
        write!(file, "{}}}", make_indent(indent - 1))?;
        Ok(())
    }
    fn generate_dts_unwrapped(&self, file: &mut File, indent: usize) -> anyhow::Result<()> {
        for field in &self.fields {
            field.generate_dts(file, indent)?;
        }
        Ok(())
    }
}
impl GenerateDTS for parser::Enum {
    fn generate_dts(&self, file: &mut File, indent: usize) -> anyhow::Result<()> {
        self.generate_dts_unwrapped(file, indent)
    }
    fn generate_dts_unwrapped(&self, file: &mut File, indent: usize) -> anyhow::Result<()> {
        if let Some(tag) = self.tag.as_ref() {
            let mut first = true;
            for (variant_name, docs, ty) in &self.variants {
                if !first {
                    write!(file, " | ")?;
                }
                writeln!(file, "{{")?;
                parser::Field {
                    docs: docs.to_owned(),
                    name: tag.to_owned(),
                    ty: parser::Type::StringLiteral(variant_name.to_owned()),
                    default: false,
                }
                .generate_dts(file, indent)?;
                if let Some(ty) = ty {
                    ty.generate_dts_unwrapped(file, indent)?;
                }
                write!(file, "{}}}", make_indent(indent - 1))?;

                first = false;
            }
        } else {
            let mut first = true;
            let indent_str = make_indent(indent);
            for (variant_name, docs, ty) in &self.variants {
                if !first {
                    write!(file, " | ")?;
                }
                writeln!(file)?;
                if let Some(ty) = ty {
                    print_doc_comment(file, &indent_str, docs)?;
                    write!(file, "{indent_str}{{{variant_name:?}: ")?;
                    ty.generate_dts(file, indent)?;
                    write!(file, "}}")?;
                } else {
                    print_doc_comment(file, &indent_str, docs)?;
                    write!(file, "{indent_str}{variant_name:?}")?;
                }

                first = false;
            }
        }
        Ok(())
    }
}
impl GenerateDTS for parser::Field {
    fn generate_dts(&self, file: &mut File, indent: usize) -> anyhow::Result<()> {
        self.generate_dts_unwrapped(file, indent)
    }

    fn generate_dts_unwrapped(&self, file: &mut File, indent: usize) -> anyhow::Result<()> {
        let indent_str = make_indent(indent);
        print_doc_comment(file, &indent_str, &self.docs)?;

        let mut default = self.default;
        let mut ty = &self.ty;
        if let parser::Type::Option(inner_ty) = ty {
            ty = inner_ty.as_ref();
            default = true;
        }

        write!(file, "{indent_str}{}{}: ", self.name, if default { "?" } else { "" })?;
        ty.generate_dts(file, indent + 1)?;
        writeln!(file, ",")?;

        Ok(())
    }
}

fn print_doc_comment(file: &mut File, indent_str: &str, docs: &str) -> anyhow::Result<()> {
    if !docs.is_empty() {
        for line in docs.lines() {
            writeln!(file, "{indent_str}/// {line}")?;
        }
    }
    Ok(())
}

fn make_indent(indent: usize) -> String {
    "  ".repeat(indent)
}

mod helpers {
    use rustdoc_types::{
        Constant as RdConstant, Crate, Id, Impl as RdImpl, Item, ItemEnum, Path as RdPath, Struct as RdStruct, Type as RdType,
        Variant as RdVariant,
    };

    use crate::{CRATES, PATH_TO_CRATE_AND_ID};

    pub trait ItemHelpers {
        fn to_struct(&self) -> Option<&RdStruct>;
        fn to_impl(&self) -> Option<&RdImpl>;
        fn to_constant(&self) -> Option<&RdConstant>;
        fn to_struct_field_type(&self) -> Option<&RdType>;
        fn to_assoc_const(&self) -> Option<&str>;
        fn to_variant(&self) -> Option<&RdVariant>;
    }
    impl ItemHelpers for ItemEnum {
        fn to_struct(&self) -> Option<&RdStruct> {
            match self {
                ItemEnum::Struct(s) => Some(s),
                _ => None,
            }
        }
        fn to_impl(&self) -> Option<&RdImpl> {
            match self {
                ItemEnum::Impl(s) => Some(s),
                _ => None,
            }
        }
        fn to_constant(&self) -> Option<&RdConstant> {
            match self {
                ItemEnum::Constant(s) => Some(s),
                _ => None,
            }
        }
        fn to_struct_field_type(&self) -> Option<&RdType> {
            match self {
                ItemEnum::StructField(s) => Some(s),
                _ => None,
            }
        }
        fn to_assoc_const(&self) -> Option<&str> {
            match self {
                ItemEnum::AssocConst { default, .. } => default.as_deref(),
                _ => None,
            }
        }
        fn to_variant(&self) -> Option<&RdVariant> {
            match self {
                ItemEnum::Variant(s) => Some(s),
                _ => None,
            }
        }
    }

    pub trait ItemsHelpers {
        fn find_named_item<'a>(&self, krate: &'a Crate, name: &str) -> Option<&'a Item>;
    }
    impl ItemsHelpers for [Id] {
        fn find_named_item<'a>(&self, krate: &'a Crate, name: &str) -> Option<&'a Item> {
            self.iter().find_map(|i| {
                let item = i.get(krate);
                if item.name.as_deref() == Some(name) {
                    Some(item)
                } else {
                    None
                }
            })
        }
    }

    pub trait IdHelper {
        fn get<'a>(&self, krate: &'a Crate) -> &'a Item;
    }
    impl IdHelper for Id {
        fn get<'a>(&self, krate: &'a Crate) -> &'a Item {
            match krate.index.get(self) {
                Some(i) => i,
                None => panic!("invalid id: {self:?}"),
            }
        }
    }

    pub trait PathHelper {
        fn get<'a>(&self, krate: &'a Crate) -> (&'a Crate, &'a Item);
    }
    impl PathHelper for RdPath {
        fn get<'a>(&self, krate: &'a Crate) -> (&'a Crate, &'a Item) {
            match krate.index.get(&self.id) {
                Some(i) => (krate, i),
                None => {
                    let path = krate.paths.get(&self.id).unwrap().path.join("::");
                    if let Some((crate_path, id)) = PATH_TO_CRATE_AND_ID.get(&path) {
                        let krate = CRATES.get(crate_path).unwrap();
                        (krate, id.get(krate))
                    } else {
                        panic!("invalid path: {self:?} {path:?}")
                    }
                }
            }
        }
    }
}

mod parser {
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
        pub variants: Vec<(String, String, Option<Type>)>,
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
                        Some(Type::Struct(Struct { name: name.clone(), fields: convert_plain_fields(krate, fields) }))
                    }
                };

                (item.name.clone().unwrap(), item.docs.clone().unwrap_or_default(), ty)
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
}
