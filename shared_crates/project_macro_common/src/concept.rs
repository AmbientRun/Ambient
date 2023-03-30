use super::{
    component::type_to_token_stream,
    tree::{Tree, TreeNode},
    util, Context,
};
use ambient_project::{
    Component, ComponentType, Concept, Identifier, IdentifierPath, IdentifierPathBuf,
};
use anyhow::Context as AnyhowContext;
use proc_macro2::TokenStream;
use quote::quote;

pub fn tree_to_token_stream(
    concept_tree: &Tree<Concept>,
    components_tree: &Tree<Component>,
    context: &Context,
) -> anyhow::Result<proc_macro2::TokenStream> {
    to_token_stream(
        concept_tree.root(),
        context,
        &match context {
            Context::Host => quote! {},
            Context::Guest { api_path, .. } => quote! {
                use super::components;
                use #api_path::prelude::*;
            },
        },
        concept_tree,
        components_tree,
    )
}

fn to_token_stream(
    node: &TreeNode<Concept>,
    context: &Context,
    prelude: &TokenStream,
    concept_tree: &Tree<Concept>,
    components_tree: &Tree<Component>,
) -> anyhow::Result<proc_macro2::TokenStream> {
    util::tree_to_token_stream(
        node,
        context,
        prelude,
        |node, context, prelude| {
            to_token_stream(node, context, prelude, concept_tree, components_tree)
        },
        |name, concept, context| {
            let make_concept =
                generate_make(concept_tree, components_tree, context, name, concept)?;
            let is_concept = generate_is(concept_tree, components_tree, context, name, concept)?;
            Ok(quote! {
                #make_concept
                #is_concept
            })
        },
    )
}

fn generate_make(
    concept_tree: &Tree<Concept>,
    component_tree: &Tree<Component>,
    context: &Context,
    name: &str,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let make_comment = format!(
        "Makes a *{}*.\n\n{}\n\n{}",
        concept.name,
        concept.description,
        generate_component_list_doc_comment(concept_tree, component_tree, context, concept)?
    );
    let make_ident = quote::format_ident!("make_{}", name);

    let extends: Vec<_> = concept
        .extends
        .iter()
        .map(|i| {
            let (last, namespaces) = i.split_last().unwrap();
            let extend_ident = quote::format_ident!("make_{}", last.as_ref());
            let supers = namespaces.iter().map(|_| quote! { super });
            quote! {
                with_merge(#(#supers::)* #(#namespaces::)* #extend_ident())
            }
        })
        .collect();

    let components_prefix = Identifier::new("components").map_err(anyhow::Error::msg)?;
    let components = concept
        .components
        .iter()
        .map(|component| {
            let full_path = build_component_path(&components_prefix, component.0.as_path());

            let manifest_component = component_tree
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
        #[allow(clippy::approx_constant)]
        #[doc = #make_comment]
        pub fn #make_ident() -> Entity {
            Entity::new()
                #(.#extends)*
                #(.#components)*
        }
    })
}

fn generate_is(
    concept_tree: &Tree<Concept>,
    component_tree: &Tree<Component>,
    context: &Context,
    name: &str,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let is_comment = format!(
        "Checks if the entity is a *{}*.\n\n{}\n\n{}",
        concept.name,
        concept.description,
        generate_component_list_doc_comment(concept_tree, component_tree, context, concept)?,
    );
    let is_ident = quote::format_ident!("is_{}", name);

    let extends: Vec<_> = concept
        .extends
        .iter()
        .map(|i| {
            let (last, namespaces) = i.split_last().unwrap();
            let extend_ident = quote::format_ident!("is_{}", last.as_ref());
            let supers = namespaces.iter().map(|_| quote! { super });
            quote! {
                #(#supers::)* #(#namespaces::)* #extend_ident(id)
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

fn build_component_path(prefix: &Identifier, path: IdentifierPath) -> IdentifierPathBuf {
    IdentifierPathBuf::from_iter(std::iter::once(prefix).chain(path.iter()).cloned())
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
            quote! { Mat4::from_cols_array(&[#arr]) }
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

pub fn generate_component_list_doc_comment(
    concept_tree: &Tree<Concept>,
    component_tree: &Tree<Component>,
    context: &Context,
    concept: &Concept,
) -> anyhow::Result<String> {
    let mut output = "*Definition*:\n\n```\n{\n".to_string();

    fn write_level(
        concepts: &Tree<Concept>,
        components: &Tree<Component>,
        context: &Context,
        concept: &Concept,
        output: &mut String,
        level: usize,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let padding = " ".repeat(level * 2);
        for (component_path, value) in &concept.components {
            let ty = components
                .get(component_path.as_path())
                .with_context(|| format!("no definition found for {component_path}"))?
                .type_
                .clone();

            writeln!(
                output,
                "{padding}\"{component_path}\": {} = {},",
                SemiprettyTokenStream(type_to_token_stream(&ty, context, false)?),
                SemiprettyTokenStream(toml_value_to_tokens(component_path.as_path(), &ty, value)?)
            )?;
        }
        for concept_path in &concept.extends {
            let concept = concepts
                .get(concept_path.as_path())
                .with_context(|| format!("no definition found for {concept_path}"))?;

            writeln!(output, "{padding}\"{concept_path}\": {{ // Concept.")?;
            write_level(concepts, components, context, concept, output, level + 1)?;
            writeln!(output, "{padding}}},")?;
        }

        Ok(())
    }

    write_level(
        concept_tree,
        component_tree,
        &match context {
            Context::Host => Context::Host,
            Context::Guest { api_path, .. } => Context::Guest {
                api_path: api_path.clone(),
                fully_qualified_path: false,
            },
        },
        concept,
        &mut output,
        1,
    )?;

    output += "}\n```\n";

    Ok(output)
}

/// Very, very basic one-line formatter for token streams
struct SemiprettyTokenStream(proc_macro2::TokenStream);
impl std::fmt::Display for SemiprettyTokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for token in self.0.clone() {
            match &token {
                proc_macro2::TokenTree::Group(g) => {
                    let (open, close) = match g.delimiter() {
                        proc_macro2::Delimiter::Parenthesis => ("(", ")"),
                        proc_macro2::Delimiter::Brace => ("{", "}"),
                        proc_macro2::Delimiter::Bracket => ("[", "]"),
                        proc_macro2::Delimiter::None => ("", ""),
                    };

                    f.write_str(open)?;
                    SemiprettyTokenStream(g.stream()).fmt(f)?;
                    f.write_str(close)?
                }
                proc_macro2::TokenTree::Punct(p) => {
                    token.fmt(f)?;
                    if p.as_char() == ',' {
                        write!(f, " ")?;
                    }
                }
                _ => token.fmt(f)?,
            }
        }
        Ok(())
    }
}
