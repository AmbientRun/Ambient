use ambient_package_semantic::{Concept, Item, ItemMap, ScalarValue, Scope, Value};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{Context, TypePrinter};

pub fn generate(
    context: Context,
    items: &ItemMap,
    type_printer: &TypePrinter,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let concepts = scope
        .concepts
        .values()
        .filter_map(|c| context.extract_item_if_relevant(items, *c))
        .map(|concept| {
            let concept = &*concept;
            let make_concept = generate_make(items, type_printer, context, concept)?;
            let is_concept = generate_is(items, type_printer, context, concept)?;
            let concept_fn = generate_concept(items, type_printer, context, concept)?;
            Ok(quote! {
                #make_concept
                #is_concept
                #concept_fn
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    if concepts.is_empty() {
        return Ok(quote! {});
    }

    let inner = match context {
        Context::Host => quote! {
            use glam::{Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, IVec2, IVec3, IVec4, Mat4, Quat};
            use crate::{EntityId, Entity, Component};
            #(#concepts)*
        },

        Context::GuestApi | Context::GuestUser => {
            let api_path = context.guest_api_path().unwrap();
            quote! {
                use #api_path::prelude::*;
                #(#concepts)*
            }
        }
    };

    Ok(quote! {
        /// Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept.
        ///
        /// They do not have any runtime representation outside of the components that compose them.
        pub mod concepts {
            #inner
        }
    })
}

fn generate_make(
    items: &ItemMap,
    type_printer: &TypePrinter,
    context: Context,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let name = concept.data().id.as_str();
    let make_comment = format!(
        "Makes a *{}*.\n\n{}\n\n{}",
        concept.name.as_deref().unwrap_or(name),
        concept.description.as_ref().unwrap_or(&"".to_string()),
        generate_component_list_doc_comment(items, type_printer, context, concept)?
    );
    let make_ident = quote::format_ident!("make_{}", name);

    let extends: Vec<_> = concept
        .extends
        .iter()
        .map(|id| {
            let path = context.get_path(items, Some("make_"), id.as_resolved().unwrap())?;

            Ok(quote! {
                with_merge(#path())
            })
        })
        .collect::<anyhow::Result<_>>()?;

    let components = concept
        .components
        .iter()
        .map(|(id, default)| {
            let path = context.get_path(items, None, id.as_resolved().unwrap())?;
            let default = value_to_token_stream(items, default.as_resolved().unwrap())?;
            Ok(quote! { with(#path(), #default) })
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
    type_printer: &TypePrinter,
    context: Context,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let name = concept.data().id.as_str();
    let is_comment = format!(
        "Checks if the entity is a *{}*.\n\n{}\n\n{}",
        concept.name.as_deref().unwrap_or(name),
        concept.description.as_ref().unwrap_or(&"".to_string()),
        generate_component_list_doc_comment(items, type_printer, context, concept)?,
    );
    let is_ident = quote::format_ident!("is_{}", name);

    let extends: Vec<_> = concept
        .extends
        .iter()
        .map(|id| context.get_path(items, Some("is_"), id.as_resolved().unwrap()))
        .collect::<anyhow::Result<_>>()?;

    let components = concept
        .components
        .iter()
        .map(|(id, _)| {
            let path = context.get_path(items, None, id.as_resolved().unwrap())?;
            Ok(quote! { #path() })
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
        Context::GuestApi | Context::GuestUser => quote! {
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
    type_printer: &TypePrinter,
    context: Context,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let name = concept.data().id.as_str();
    let fn_comment = format!(
        "Returns the components that comprise *{}* as a tuple.\n\n{}\n\n{}",
        concept.name.as_deref().unwrap_or(name),
        concept.description.as_ref().unwrap_or(&"".to_string()),
        generate_component_list_doc_comment(items, type_printer, context, concept)?,
    );
    let fn_ident = quote::format_ident!("{}", name);

    // TODO: include extends in component list
    let components = concept
        .components
        .iter()
        .map(|(id, _)| {
            let path = context.get_path(items, None, id.as_resolved().unwrap())?;
            Ok(quote! { #path() })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let fn_ret = concept
        .components
        .iter()
        .map(|(id, _)| {
            let component = &*items.get(id.as_resolved().unwrap());
            type_printer.get(context, items, None, component.type_.as_resolved().unwrap())
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(quote! {
        #[doc = #fn_comment]
        #[allow(clippy::type_complexity)]
        pub fn #fn_ident() -> (#(Component<#fn_ret>),*) {
           (#(#components),*)
        }
    })
}

pub fn generate_component_list_doc_comment(
    items: &ItemMap,
    type_printer: &TypePrinter,
    context: Context,
    concept: &Concept,
) -> anyhow::Result<String> {
    let mut output = "*Definition*:\n\n```ignore\n{\n".to_string();

    fn write_level(
        items: &ItemMap,

        type_printer: &TypePrinter,
        context: Context,
        concept: &Concept,
        output: &mut String,
        level: usize,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let padding = " ".repeat(level * 2);
        for (id, value) in &concept.components {
            let component = &*items.get(id.as_resolved().unwrap());
            let component_path = items.fully_qualified_display_path(component, None, None);

            writeln!(
                output,
                "{padding}\"{component_path}\": {} = {},",
                SemiprettyTokenStream(
                    type_printer
                        .get(context, items, None, component.type_.as_resolved().unwrap())
                        .unwrap()
                        .clone()
                ),
                value.as_resolved().unwrap()
            )?;
        }
        for concept_id in &concept.extends {
            let concept_id = concept_id.as_resolved().unwrap();
            let concept = &*items.get(concept_id);
            let concept_path = items.fully_qualified_display_path(concept, None, None);

            writeln!(output, "{padding}\"{concept_path}\": {{ // Concept.")?;
            write_level(items, type_printer, context, concept, output, level + 1)?;
            writeln!(output, "{padding}}},")?;
        }

        Ok(())
    }

    write_level(items, type_printer, context, concept, &mut output, 1)?;

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
            let streams = v.iter().map(scalar_value_to_token_stream);
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
            let item = &*items.get(*id);
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
        ScalarValue::String(v) => quote! { #v.to_string() },
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

#[cfg(test)]
mod tests {
    use ambient_package::PascalCaseIdentifier;
    use ambient_package_semantic::{
        create_root_scope, Enum, ItemData, ItemSource, Type, TypeInner,
    };

    use super::*;

    #[test]
    fn test_scalar_value_to_token_stream() {
        let v = ScalarValue::Empty(());
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "()");

        let v = ScalarValue::Bool(true);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "true");

        let v = ScalarValue::EntityId(42);
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "EntityId (42u128)"
        );

        let v = ScalarValue::F32(42.345);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "42.345f32");

        let v = ScalarValue::F64(42.345);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "42.345f64");

        let v = ScalarValue::Mat4(glam::Mat4::IDENTITY);
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "Mat4 :: from_cols_array (& [1f32 , 0f32 , 0f32 , 0f32 , 0f32 , 1f32 , 0f32 , 0f32 , 0f32 , 0f32 , 1f32 , 0f32 , 0f32 , 0f32 , 0f32 , 1f32 ,])"
        );

        let v = ScalarValue::Quat(glam::Quat::IDENTITY);
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "Quat :: from_xyzw (0f32 , 0f32 , 0f32 , 1f32 ,)"
        );

        let v = ScalarValue::String("hello".to_string());
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "\"hello\" . to_string ()"
        );

        let v = ScalarValue::U8(42);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "42u8");

        let v = ScalarValue::U16(42);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "42u16");

        let v = ScalarValue::U32(42);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "42u32");

        let v = ScalarValue::U64(42);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "42u64");

        let v = ScalarValue::I8(-42);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "- 42i8");

        let v = ScalarValue::I16(-42);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "- 42i16");

        let v = ScalarValue::I32(-42);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "- 42i32");

        let v = ScalarValue::I64(-42);
        assert_eq!(scalar_value_to_token_stream(&v).to_string(), "- 42i64");

        let v = ScalarValue::Vec2(glam::Vec2::new(1f32, 2f32));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "Vec2 :: new (1f32 , 2f32 ,)"
        );

        let v = ScalarValue::Vec3(glam::Vec3::new(1f32, 2f32, 3f32));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "Vec3 :: new (1f32 , 2f32 , 3f32 ,)"
        );

        let v = ScalarValue::Vec4(glam::Vec4::new(1f32, 2f32, 3f32, 4f32));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "Vec4 :: new (1f32 , 2f32 , 3f32 , 4f32 ,)"
        );

        let v = ScalarValue::Uvec2(glam::UVec2::new(1, 2));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "UVec2 :: new (1u32 , 2u32 ,)"
        );

        let v = ScalarValue::Uvec3(glam::UVec3::new(1, 2, 3));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "UVec3 :: new (1u32 , 2u32 , 3u32 ,)"
        );

        let v = ScalarValue::Uvec4(glam::UVec4::new(1, 2, 3, 4));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "UVec4 :: new (1u32 , 2u32 , 3u32 , 4u32 ,)"
        );

        let v = ScalarValue::Ivec2(glam::IVec2::new(-1, -2));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "IVec2 :: new (- 1i32 , - 2i32 ,)"
        );

        let v = ScalarValue::Ivec3(glam::IVec3::new(-1, -2, -3));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "IVec3 :: new (- 1i32 , - 2i32 , - 3i32 ,)"
        );

        let v = ScalarValue::Ivec4(glam::IVec4::new(-1, -2, -3, -4));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "IVec4 :: new (- 1i32 , - 2i32 , - 3i32 , - 4i32 ,)"
        );

        let v = ScalarValue::Duration(std::time::Duration::new(42, 345));
        assert_eq!(
            scalar_value_to_token_stream(&v).to_string(),
            "Duration :: new (42u64 , 345u32)"
        );

        fn unsupported_test<T: Default>(constructor: impl Fn(T) -> ScalarValue) {
            let v = constructor(Default::default());
            assert_eq!(
                scalar_value_to_token_stream(&v).to_string(),
                "unsupported ! ()"
            );
        }

        unsupported_test(ScalarValue::ProceduralMeshHandle);
        unsupported_test(ScalarValue::ProceduralTextureHandle);
        unsupported_test(ScalarValue::ProceduralSamplerHandle);
        unsupported_test(ScalarValue::ProceduralMaterialHandle);
    }

    #[test]
    fn test_value_to_token_stream() {
        let mut items = ItemMap::default();
        let _ = create_root_scope(&mut items).unwrap();

        let value = Value::Scalar(ScalarValue::Bool(true));
        assert_eq!(
            value_to_token_stream(&items, &value).unwrap().to_string(),
            "true"
        );

        let value = Value::Vec(vec![
            ScalarValue::U32(1),
            ScalarValue::U32(2),
            ScalarValue::U32(3),
        ]);
        assert_eq!(
            value_to_token_stream(&items, &value).unwrap().to_string(),
            "vec ! [1u32 , 2u32 , 3u32 ,]"
        );

        let value = Value::Option(Some(ScalarValue::String("hello".to_string())));
        assert_eq!(
            value_to_token_stream(&items, &value).unwrap().to_string(),
            "Some (\"hello\" . to_string ())"
        );

        let value = Value::Option(None);
        assert_eq!(
            value_to_token_stream(&items, &value).unwrap().to_string(),
            "None"
        );

        let id = items.add(Type::new(
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
        let value = Value::Enum(id, PascalCaseIdentifier::new("B").unwrap());
        assert_eq!(
            value_to_token_stream(&items, &value).unwrap().to_string(),
            "1usize"
        );
    }
}
