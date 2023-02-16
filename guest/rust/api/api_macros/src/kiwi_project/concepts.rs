use super::{
    components::Tree,
    identifier::{Identifier, IdentifierPath, IdentifierPathBuf},
    manifest::{ComponentType, Concept, Manifest},
};
use anyhow::Context;
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn generate_tokens(
    manifest: &Manifest,
    components_tree: &Tree,
    api_name: &syn::Path,
) -> anyhow::Result<TokenStream> {
    if manifest.concepts.is_empty() {
        return Ok(quote! {});
    }

    let concepts_tokens = manifest
        .concepts
        .iter()
        .map(|concept| {
            let make_concept = generate_make(components_tree, concept.0, concept.1)?;
            let is_concept = generate_is(concept.0, concept.1)?;

            Ok(quote! {
                #make_concept
                #is_concept
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(quote! {
        use super::components;
        use #api_name::prelude::*;

        #(#concepts_tokens)*
    })
}

fn build_component_path(prefix: &Identifier, path: IdentifierPath) -> IdentifierPathBuf {
    IdentifierPathBuf::from_iter(std::iter::once(prefix).chain(path.iter()).cloned())
}

fn generate_make(
    components_tree: &Tree,
    identifier: &Identifier,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let make_comment = format!("Makes a {} ({})", concept.name, concept.description);
    let make_ident = quote::format_ident!("make_{}", identifier.as_ref());

    let extends: Vec<_> = concept
        .extends
        .iter()
        .map(|i| {
            let extend_ident = quote::format_ident!("make_{}", i.as_ref());
            quote! {
                merge(#extend_ident())
            }
        })
        .collect();

    let components_prefix = Identifier::new("components").map_err(anyhow::Error::msg)?;
    let components = concept
        .components
        .iter()
        .map(|component| {
            let full_path = build_component_path(&components_prefix, component.0.as_path());

            let manifest_component = components_tree
                .get(component.0.as_path())
                .with_context(|| format!("there is no component defined at `{}`", component.0))?;

            let default = toml_value_to_tokens(
                component.0.as_path(),
                &manifest_component.type_,
                component.1,
            )?;

            Ok(quote! { with(#full_path(), #default) })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(quote! {
        #[doc = #make_comment]
        pub fn #make_ident() -> Components {
            Components::new()
                #(.#extends)*
                #(.#components)*
        }
    })
}

fn generate_is(identifier: &Identifier, concept: &Concept) -> anyhow::Result<TokenStream> {
    let is_comment = format!(
        "Checks if the entity is a {} ({})",
        concept.name, concept.description
    );
    let is_ident = quote::format_ident!("is_{}", identifier.as_ref());

    let extends: Vec<_> = concept
        .extends
        .iter()
        .map(|i| {
            let extend_ident = quote::format_ident!("is_{}", i.as_ref());
            quote! {
                #extend_ident(id)
            }
        })
        .collect();

    let components_prefix = Identifier::new("components").map_err(anyhow::Error::msg)?;
    let components: Vec<_> = concept
        .components
        .iter()
        .map(|c| build_component_path(&components_prefix, c.0.as_path()))
        .map(|p| quote! { #p() })
        .collect();

    Ok(quote! {
        #[doc = #is_comment]
        pub fn #is_ident(id: EntityId) -> bool {
            #(#extends && )* entity::has_components(id, &[
                #(&#components),*
            ])
        }
    })
}

fn toml_value_to_tokens(
    path: IdentifierPath,
    ty: &ComponentType,
    value: &toml::Value,
) -> anyhow::Result<TokenStream> {
    match ty {
        ComponentType::String(ty) => toml_value_to_tokens_primitive(path, ty, value),
        ComponentType::ContainerType {
            type_,
            element_type,
        } => {
            if let Some(element_type) = element_type {
                let values = value.as_array().with_context(|| {
                    format!("expected an array initializer for component `{path}`")
                })?;

                match type_.as_str() {
                    "Vec" => {
                        let values = values
                            .iter()
                            .map(|v| toml_value_to_tokens_primitive(path, element_type, v))
                            .collect::<anyhow::Result<Vec<_>>>()?;

                        Ok(quote! { vec![ #(#values),* ] })
                    }
                    "Option" => {
                        if values.is_empty() {
                            Ok(quote! { None })
                        } else {
                            let value =
                                toml_value_to_tokens_primitive(path, element_type, &values[0])?;
                            Ok(quote! { Some(#value) })
                        }
                    }
                    _ => anyhow::bail!("unsupported container `{type_}` for component `{path}`"),
                }
            } else {
                toml_value_to_tokens_primitive(path, type_, value)
            }
        }
    }
}

fn toml_value_to_tokens_primitive(
    path: IdentifierPath,
    ty: &str,
    value: &toml::Value,
) -> anyhow::Result<TokenStream> {
    Ok(match (ty, value) {
        ("Empty", toml::Value::Table(t)) if t.is_empty() => quote! {()},
        ("Bool", toml::Value::Boolean(b)) => quote! {#b},
        ("EntityId", toml::Value::String(s)) => quote! {EntityId::from_base64(#s)},
        ("F32", toml::Value::Float(f)) => {
            let f = *f as f32;
            quote! {#f}
        }
        ("F64", toml::Value::Float(f)) => {
            quote! {#f}
        }
        ("Mat4", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Mat4::from_cols_array([#arr]) }
        }
        ("I32", toml::Value::Integer(i)) => {
            let i = *i as i32;
            quote! {#i}
        }
        ("Quat", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Quat::from_xyzw(#arr) }
        }
        ("String", toml::Value::String(s)) => quote! {#s.to_string()},
        ("U32", toml::Value::Integer(i)) => {
            let i = *i as u32;
            quote! {#i}
        }
        ("U64", toml::Value::String(s)) => {
            let val: u64 = s.parse()?;
            quote! {#val}
        }
        ("Vec2", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Vec2::new(#arr) }
        }
        ("Vec3", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Vec3::new(#arr) }
        }
        ("Vec4", toml::Value::Array(a)) => {
            let arr = toml_array_f32_to_array_tokens(path, a)?;
            quote! { Vec4::new(#arr) }
        }
        ("ObjectRef", toml::Value::String(s)) => quote! {ObjectRef::new(#s)},
        _ => anyhow::bail!("unsupported type `{ty}` and value `{value}` for component `{path}`"),
    })
}

fn toml_array_f32_to_array_tokens(
    path: IdentifierPath,
    array: &toml::value::Array,
) -> anyhow::Result<TokenStream> {
    let members = array
        .iter()
        .map(|c| {
            if let Some(f) = c.as_float() {
                Ok(f as f32)
            } else if let Some(i) = c.as_integer() {
                Ok(i as f32)
            } else {
                anyhow::bail!(
                    "not all of the values for the array initializer for `{path}` were numbers"
                )
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(quote! { #(#members),* })
}
