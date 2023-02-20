//! This code is extremely ugly. Its sole purpose is to convert our serde types
//! into something that can be used in documentation.
use std::{collections::HashMap, fs::File};

use anyhow::Context;
use helpers::*;
use once_cell::sync::Lazy;
use rustdoc_types::{Crate, Id};
use std::io::Write;

mod helpers;
mod parser;

static CRATES: Lazy<HashMap<String, Crate>> = Lazy::new(|| {
    ["crates/physics/Cargo.toml", "crates/model_import/Cargo.toml", "crates/build/Cargo.toml"]
        .iter()
        .map(|n| {
            let build =
                rustdoc_json::Builder::default().toolchain("nightly").document_private_items(true).manifest_path(n).silent(true).build()?;
            let krate = serde_json::from_str(&std::fs::read_to_string(build)?)?;
            anyhow::Ok((n.to_string(), krate))
        })
        .collect::<anyhow::Result<_>>()
        .unwrap()
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
    let pipeline = id.get(&build_crate);
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
        Ok(match self {
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
        })
    }
    fn generate_dts_unwrapped(&self, file: &mut File, indent: usize) -> anyhow::Result<()> {
        Ok(match self {
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
        })
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
    Ok(if !docs.is_empty() {
        for line in docs.lines() {
            writeln!(file, "{indent_str}/// {line}")?;
        }
    })
}

fn make_indent(indent: usize) -> String {
    "  ".repeat(indent)
}
