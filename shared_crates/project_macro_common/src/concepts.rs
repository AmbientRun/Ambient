use std::{collections::HashMap, str::FromStr};

use ambient_project_semantic::{Concept, Item, ItemId, ItemMap, ResolvedValue, Scope, Type};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{make_path, Context};

pub fn make_definitions(
    context: &Context,
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, TokenStream>,
    _root_scope_id: ItemId<Scope>,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let scopes = scope
        .scopes
        .values()
        .map(|s| {
            let scope = items.get(*s)?;
            let id = make_path(&scope.data.id.as_snake_case());

            let inner = make_definitions(context, items, type_map, _root_scope_id, &scope)?;
            if inner.is_empty() {
                return Ok(quote! {});
            }
            Ok(quote! {
                #[allow(unused)]
                pub mod #id {
                    #inner
                }
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let concepts = scope
        .concepts
        .values()
        .map(|id| {
            let concept = &*items.get(*id)?;
            let make_concept = generate_make(items, type_map, context, concept)?;
            let is_concept = generate_is(items, type_map, context, concept)?;
            let concept_fn = generate_concept(items, type_map, context, concept)?;
            Ok(quote! {
                #make_concept
                #is_concept
                #concept_fn
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let inner = if concepts.is_empty() {
        quote! {}
    } else {
        match context {
            Context::Host => quote! {
                use glam::{Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, IVec2, IVec3, IVec4, Mat4, Quat};
                use crate::{EntityId, Entity, Component};
                #(#concepts)*
            },

            Context::Guest { api_path, .. } => quote! {
                use #api_path::prelude::*;
                #(#concepts)*
            },
        }
    };

    Ok(quote! {
        #(#scopes)*
        #inner
    })
}

fn generate_make(
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, TokenStream>,
    context: &Context,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let name = concept.data().id.as_snake_case();
    let make_comment = format!(
        "Makes a *{}*.\n\n{}\n\n{}",
        concept.name.as_deref().unwrap_or(&name),
        concept.description.as_ref().unwrap_or(&"".to_string()),
        generate_component_list_doc_comment(items, type_map, context, concept)?
    );
    let make_ident = quote::format_ident!("make_{}", name);

    let extends: Vec<_> = concept
        .extends
        .iter()
        .map(|id| {
            let concept = &*items.get(id.as_resolved().unwrap())?;
            let extend_path =
                make_path(&items.fully_qualified_display_path_rust_style(concept, None)?);
            Ok(quote! {
                with_merge(crate :: concepts :: #extend_path())
            })
        })
        .collect::<anyhow::Result<_>>()?;

    let components = concept
        .components
        .iter()
        .map(|(id, default)| {
            let component = &*items.get(id.as_resolved().unwrap())?;
            let full_path =
                make_path(&items.fully_qualified_display_path_rust_style(component, None)?);
            let default = value_to_token_stream(items, default.as_resolved().unwrap())?;

            Ok(quote! { with(crate :: components :: #full_path(), #default) })
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
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, TokenStream>,
    context: &Context,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let name = concept.data().id.as_snake_case();
    let is_comment = format!(
        "Checks if the entity is a *{}*.\n\n{}\n\n{}",
        concept.name.as_deref().unwrap_or(&name),
        concept.description.as_ref().unwrap_or(&"".to_string()),
        generate_component_list_doc_comment(items, type_map, context, concept)?,
    );
    let is_ident = quote::format_ident!("is_{}", name);

    let extends: Vec<_> = concept
        .extends
        .iter()
        .map(|id| {
            let concept = &*items.get(id.as_resolved().unwrap())?;
            let extend_path =
                make_path(&items.fully_qualified_display_path_rust_style(concept, None)?);
            Ok(quote! {
                crate :: concepts :: #extend_path()
            })
        })
        .collect::<anyhow::Result<_>>()?;

    let components = concept
        .components
        .iter()
        .map(|(id, _)| {
            let component = &*items.get(id.as_resolved().unwrap())?;
            let full_path =
                make_path(&items.fully_qualified_display_path_rust_style(component, None)?);

            Ok(quote! { crate :: components :: #full_path() })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

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
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, TokenStream>,
    context: &Context,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let name = concept.data().id.as_snake_case();
    let fn_comment = format!(
        "Returns the components that comprise *{}* as a tuple.\n\n{}\n\n{}",
        concept.name.as_deref().unwrap_or(&name),
        concept.description.as_ref().unwrap_or(&"".to_string()),
        generate_component_list_doc_comment(items, type_map, context, concept)?,
    );
    let fn_ident = quote::format_ident!("{}", name);

    let components = concept
        .components
        .iter()
        .map(|(id, _)| {
            let component = &*items.get(id.as_resolved().unwrap())?;
            let full_path =
                make_path(&items.fully_qualified_display_path_rust_style(component, None)?);

            Ok(quote! { crate :: components :: #full_path() })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let fn_ret = concept
        .components
        .iter()
        .map(|(id, _)| {
            let component = &*items.get(id.as_resolved().unwrap())?;
            Ok(type_map
                .get(&component.type_.as_resolved().unwrap())
                .cloned()
                .unwrap())
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(quote! {
        #[doc = #fn_comment]
        pub fn #fn_ident() -> (#(Component<#fn_ret>),*) {
           (#(#components),*)
        }
    })
}

pub fn generate_component_list_doc_comment(
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, TokenStream>,
    context: &Context,
    concept: &Concept,
) -> anyhow::Result<String> {
    let mut output = "*Definition*:\n\n```ignore\n{\n".to_string();

    fn write_level(
        items: &ItemMap,
        type_map: &HashMap<ItemId<Type>, TokenStream>,
        context: &Context,
        concept: &Concept,
        output: &mut String,
        level: usize,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let padding = " ".repeat(level * 2);
        for (id, value) in &concept.components {
            let component = &*items.get(id.as_resolved().unwrap())?;
            let component_path =
                items.fully_qualified_display_path_ambient_style(component, false, None)?;

            writeln!(
                output,
                "{padding}\"{component_path}\": {} = {},",
                SemiprettyTokenStream(
                    type_map
                        .get(&component.type_.as_resolved().unwrap())
                        .unwrap()
                        .clone()
                ),
                value.as_resolved().unwrap()
            )?;
        }
        for concept_id in &concept.extends {
            let concept_id = concept_id.as_resolved().unwrap();
            let concept = &*items.get(concept_id)?;
            let concept_path =
                items.fully_qualified_display_path_ambient_style(concept, false, None)?;

            writeln!(output, "{padding}\"{concept_path}\": {{ // Concept.")?;
            write_level(items, type_map, context, concept, output, level + 1)?;
            writeln!(output, "{padding}}},")?;
        }

        Ok(())
    }

    write_level(
        items,
        type_map,
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
struct SemiprettyTokenStream(TokenStream);
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

fn value_to_token_stream(items: &ItemMap, value: &Value) -> anyhow::Result<TokenStream> {
    Ok(match value {
        Value::Scalar(v) => scalar_value_to_token_stream(v),
        Value::Vec(v) => {
            let streams = v.iter().map(|v| scalar_value_to_token_stream(v));
            quote! { vec![#(#streams,)*] }
        }
        Value::Option(v) => match v.as_ref() {
            Some(v) => {
                let v = scalar_value_to_token_stream(v);
                quote! { Some(#v) }
            }
            None => quote! { None },
        },
        Value::Enum(id, member) => {
            let item = &*items.get(*id)?;
            let index = item
                .inner
                .as_enum()
                .unwrap()
                .members
                .get_index_of(member)
                .unwrap();

            quote! { #index }
        }
    })
}

fn scalar_value_to_token_stream(v: &ScalarValue) -> TokenStream {
    match v {
        ScalarValue::Empty(_) => quote! { () },
        ScalarValue::Bool(v) => quote! { #v },
        ScalarValue::EntityId(id) => quote! { EntityId(#id) },
        ScalarValue::F32(v) => quote! { #v },
        ScalarValue::F64(v) => quote! { #v },
        ScalarValue::Mat4(v) => {
            let arr = v.to_cols_array();
            quote! { Mat4::from_cols_array(&[#(#arr,)*]) }
        }
        ScalarValue::Quat(v) => {
            let arr = v.to_array();
            quote! { Quat::from_xyzw(#(#arr,)*) }
        }
        ScalarValue::String(v) => quote! { #v },
        ScalarValue::U8(v) => quote! { #v },
        ScalarValue::U16(v) => quote! { #v },
        ScalarValue::U32(v) => quote! { #v },
        ScalarValue::U64(v) => quote! { #v },
        ScalarValue::I8(v) => quote! { #v },
        ScalarValue::I16(v) => quote! { #v },
        ScalarValue::I32(v) => quote! { #v },
        ScalarValue::I64(v) => quote! { #v },
        ScalarValue::Vec2(v) => {
            let arr = v.to_array();
            quote! { Vec2::new(#(#arr,)*) }
        }
        ScalarValue::Vec3(v) => {
            let arr = v.to_array();
            quote! { Vec3::new(#(#arr,)*) }
        }
        ScalarValue::Vec4(v) => {
            let arr = v.to_array();
            quote! { Vec4::new(#(#arr,)*) }
        }
        ScalarValue::Uvec2(v) => {
            let arr = v.to_array();
            quote! { UVec2::new(#(#arr,)*) }
        }
        ScalarValue::Uvec3(v) => {
            let arr = v.to_array();
            quote! { UVec3::new(#(#arr,)*) }
        }
        ScalarValue::Uvec4(v) => {
            let arr = v.to_array();
            quote! { UVec4::new(#(#arr,)*) }
        }
        ScalarValue::Ivec2(v) => {
            let arr = v.to_array();
            quote! { IVec2::new(#(#arr,)*) }
        }
        ScalarValue::Ivec3(v) => {
            let arr = v.to_array();
            quote! { IVec3::new(#(#arr,)*) }
        }
        ScalarValue::Ivec4(v) => {
            let arr = v.to_array();
            quote! { IVec4::new(#(#arr,)*) }
        }
        ScalarValue::Duration(v) => {
            let secs = v.as_secs();
            let nanos = v.subsec_nanos();
            quote! { Duration::new(#secs, #nanos) }
        }
        ScalarValue::ProceduralMeshHandle(_v) => quote! { unsupported!() },
        ScalarValue::ProceduralTextureHandle(_v) => quote! { unsupported!() },
        ScalarValue::ProceduralSamplerHandle(_v) => quote! { unsupported!() },
        ScalarValue::ProceduralMaterialHandle(_v) => quote! { unsupported!() },
    }
}
