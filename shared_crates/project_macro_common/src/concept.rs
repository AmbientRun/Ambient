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
        |context, _ns, ts| match context {
            Context::Host => quote! {
                use super::components;
                use glam::{Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, Mat4, Quat};
                use crate::{EntityId, Entity, Component};
                #ts
            },
            Context::Guest { api_path, .. } => quote! {
                use super::components;
                use #api_path::prelude::*;
                #ts
            },
        },
        concept_tree,
        components_tree,
    )
}

fn to_token_stream(
    node: &TreeNode<Concept>,
    context: &Context,
    wrapper: impl Fn(&Context, &TreeNode<Concept>, TokenStream) -> TokenStream + Copy,
    concept_tree: &Tree<Concept>,
    components_tree: &Tree<Component>,
) -> anyhow::Result<proc_macro2::TokenStream> {
    util::tree_to_token_stream(
        node,
        context,
        wrapper,
        |node, context, wrapper| {
            to_token_stream(node, context, wrapper, concept_tree, components_tree)
        },
        |name, concept, context| {
            let make_concept =
                generate_make(concept_tree, components_tree, context, name, concept)?;
            let is_concept = generate_is(concept_tree, components_tree, context, name, concept)?;
            let concept_fn =
                generate_concept(concept_tree, components_tree, context, name, concept)?;
            Ok(quote! {
                #make_concept
                #is_concept
                #concept_fn
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
        concept.name.as_ref().map(|x| x as &str).unwrap_or(name),
        concept.description.as_ref().unwrap_or(&"".to_string()),
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
        concept.name.as_ref().map(|x| x as &str).unwrap_or(name),
        concept.description.as_ref().unwrap_or(&"".to_string()),
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
                #(#supers::)* #(#namespaces::)* #extend_ident
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

    Ok(match context {
        Context::Host => quote! {
            #[doc = #is_comment]
            pub fn #is_ident(world: &crate::World, id: EntityId) -> bool {
                #(#extends(world, id) && )* world.has_components(id, &{
                    let mut set = crate::ComponentSet::new();
                    #(set.insert(#components.desc());)*
                    set
                })
            }
        },
        Context::Guest { .. } => quote! {
            #[doc = #is_comment]
            pub fn #is_ident(id: EntityId) -> bool {
                #(#extends(id) && )* entity::has_components(id, &[
                    #(&#components),*
                ])
            }
        },
    })
}

fn generate_concept(
    concept_tree: &Tree<Concept>,
    component_tree: &Tree<Component>,
    context: &Context,
    name: &str,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let fn_comment = format!(
        "Returns the components that comprise *{}* as a tuple.\n\n{}\n\n{}",
        concept.name.as_ref().map(|x| x as &str).unwrap_or(name),
        concept.description.as_ref().unwrap_or(&"".to_string()),
        generate_component_list_doc_comment(concept_tree, component_tree, context, concept)?,
    );
    let fn_ident = quote::format_ident!("{}", name);

    let components_prefix = Identifier::new("components").map_err(anyhow::Error::msg)?;

    fn concept_to_component<'a>(
        concept_tree: &'a Tree<Concept>,
        component_tree: &'a Tree<Component>,
        concept: &'a Concept,
    ) -> anyhow::Result<Vec<(IdentifierPath<'a>, &'a Component)>> {
        let mut result = vec![];

        for concept_path in &concept.extends {
            result.append(&mut concept_to_component(
                concept_tree,
                component_tree,
                concept_tree.get(concept_path.as_path()).with_context(|| {
                    format!(
                        "there is no concept defined at `{}`",
                        concept_path.as_path()
                    )
                })?,
            )?);
        }

        for component_path in concept.components.keys() {
            let path = component_path.as_path();
            result.push((
                path,
                component_tree.get(path).with_context(|| {
                    format!(
                        "there is no component defined at `{}`",
                        component_path.as_path()
                    )
                })?,
            ));
        }

        Ok(result)
    }

    let components = concept_to_component(concept_tree, component_tree, concept)?;
    let component_fns = components
        .iter()
        .map(|c| build_component_path(&components_prefix, c.0))
        .map(|p| quote! { #p() });

    let fn_ret = components
        .iter()
        .map(|c| Ok(type_to_token_stream(&c.1.type_, context, false)?))
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(quote! {
        #[doc = #fn_comment]
        pub fn #fn_ident() -> (#(Component<#fn_ret>),*) {
           (#(#component_fns),*)
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
            let values = value
                .as_array()
                .with_context(|| format!("expected an array initializer for component `{path}`"))?;

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
                        let value = toml_value_to_tokens_primitive(path, element_type, &values[0])?;
                        Ok(quote! { Some(#value) })
                    }
                }
                _ => anyhow::bail!("unsupported container `{type_}` for component `{path}`"),
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
    let mut output = "*Definition*:\n\n```ignore\n{\n".to_string();

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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ambient_project::{Component, ComponentType, Concept, IdentifierPathBuf};

    use crate::{tests::guest_context, tree::Tree};

    #[test]
    fn can_generate_nested_doc_comment_for_concepts() {
        let component_tree = Tree::<Component>::new(
            &BTreeMap::from_iter(
                ["Bool", "F32", "I32", "String", "Vec3"]
                    .into_iter()
                    .enumerate()
                    .map(|(idx, ty)| {
                        (
                            IdentifierPathBuf::new(format!("component{idx}")).unwrap(),
                            Component {
                                name: Some(format!("Component {idx}")),
                                description: None,
                                type_: ComponentType::String(ty.to_string()),
                                attributes: vec![],
                                default: None,
                            }
                            .into(),
                        )
                    }),
            ),
            false,
        )
        .unwrap();

        let concept_tree = Tree::<Concept>::new(
            &BTreeMap::from_iter([
                (
                    IdentifierPathBuf::new("concept0").unwrap(),
                    Concept {
                        name: None,
                        description: None,
                        extends: vec![],
                        components: BTreeMap::from_iter([(
                            IdentifierPathBuf::new("component0").unwrap(),
                            toml::Value::Boolean(true),
                        )]),
                    }
                    .into(),
                ),
                (
                    IdentifierPathBuf::new("concept1").unwrap(),
                    Concept {
                        name: None,
                        description: None,
                        extends: vec![IdentifierPathBuf::new("concept0").unwrap()],
                        components: BTreeMap::from_iter([(
                            IdentifierPathBuf::new("component1").unwrap(),
                            toml::Value::Float(4.56),
                        )]),
                    }
                    .into(),
                ),
                (
                    IdentifierPathBuf::new("concept2").unwrap(),
                    Concept {
                        name: None,
                        description: None,
                        extends: vec![],
                        components: BTreeMap::from_iter([(
                            IdentifierPathBuf::new("component2").unwrap(),
                            toml::Value::Integer(3),
                        )]),
                    }
                    .into(),
                ),
                (
                    IdentifierPathBuf::new("concept3").unwrap(),
                    Concept {
                        name: None,
                        description: None,
                        extends: vec![
                            IdentifierPathBuf::new("concept1").unwrap(),
                            IdentifierPathBuf::new("concept2").unwrap(),
                        ],
                        components: BTreeMap::from_iter([
                            (
                                IdentifierPathBuf::new("component3").unwrap(),
                                toml::Value::String("It's pi".to_string()),
                            ),
                            (
                                IdentifierPathBuf::new("component4").unwrap(),
                                toml::Value::Array((0..3).map(toml::Value::Integer).collect()),
                            ),
                        ]),
                    }
                    .into(),
                ),
            ]),
            false,
        )
        .unwrap();

        let comment = super::generate_component_list_doc_comment(
            &concept_tree,
            &component_tree,
            &guest_context(),
            concept_tree
                .get(IdentifierPathBuf::new("concept3").unwrap().as_path())
                .unwrap(),
        )
        .unwrap();

        assert_eq!(
            comment,
            indoc::indoc! {r#"
            *Definition*:

            ```ignore
            {
              "component3": String = "It's pi".to_string(),
              "component4": Vec3 = Vec3::new(0f32, 1f32, 2f32),
              "concept1": { // Concept.
                "component1": f32 = 4.56f32,
                "concept0": { // Concept.
                  "component0": bool = true,
                },
              },
              "concept2": { // Concept.
                "component2": i32 = 3i32,
              },
            }
            ```
        "#}
        );
    }
}
