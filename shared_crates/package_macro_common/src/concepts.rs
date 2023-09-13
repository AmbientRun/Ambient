use std::fmt::Write;

use ambient_package::Identifier;
use ambient_package_semantic::{
    Component, Concept, ConceptValue, Item, ItemId, ItemMap, ScalarValue, Scope, Value,
};

use proc_macro2::TokenStream;
use quote::quote;

use crate::{Context, TypePrinter};

pub fn generate(
    context: Context,
    items: &ItemMap,
    type_printer: &TypePrinter,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let Some(guest_api_path) = context.guest_api_path() else {
        // Concept generation is not supported on the host.
        return Ok(quote! {});
    };

    let concepts = scope
        .concepts
        .values()
        .filter_map(|c| context.extract_item_if_relevant(items, *c))
        .map(|concept| {
            let concept = &*concept;
            let new = generate_one(items, type_printer, context, concept)?;
            Ok(quote! {
                #new
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    if concepts.is_empty() {
        return Ok(quote! {});
    }

    Ok(quote! {
        /// Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept.
        ///
        /// They do not have any runtime representation outside of the components that compose them.
        pub mod concepts {
            use #guest_api_path::prelude::*;
            #(#concepts)*
        }
    })
}

fn generate_one(
    items: &ItemMap,
    type_printer: &TypePrinter,
    context: Context,
    concept: &Concept,
) -> anyhow::Result<TokenStream> {
    let concept_id = &concept.data().id;
    let concept_optional_id = quote::format_ident!("{}Optional", concept_id.as_str());

    let required_components = concept
        .required_components
        .iter()
        .map(|(id, value)| {
            component_to_field(
                items,
                type_printer,
                context,
                id.as_resolved().unwrap(),
                value,
            )
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let optional_components = concept
        .optional_components
        .iter()
        .map(|(id, value)| {
            component_to_field(
                items,
                type_printer,
                context,
                id.as_resolved().unwrap(),
                value,
            )
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let struct_def = {
        let doc_comment = {
            let mut doc_comment = String::new();
            write!(
                doc_comment,
                "**{}**",
                concept.name.as_deref().unwrap_or(concept_id.as_str())
            )?;
            if let Some(description) = &concept.description {
                write!(doc_comment, ": {}", description)?;
            }
            writeln!(doc_comment)?;
            writeln!(doc_comment)?;

            if !concept.extends.is_empty() {
                write!(doc_comment, "**Extends**: ")?;
                for (i, id) in concept.extends.iter().enumerate() {
                    let extend = items.get(id.as_resolved().unwrap());
                    if i != 0 {
                        doc_comment.push_str(", ");
                    }

                    write!(
                        doc_comment,
                        "`{}`",
                        &items.fully_qualified_display_path(extend, None, None)
                    )?;
                }
                writeln!(doc_comment)?;
                writeln!(doc_comment)?;
            }
            doc_comment.trim().to_string()
        };

        let components = required_components
            .iter()
            .map(|component| component.to_field_definition(false));

        let optional_ref = if !optional_components.is_empty() {
            Some(quote! {
                /// Optional components.
                pub optional: #concept_optional_id,
            })
        } else {
            None
        };

        quote! {
            #[doc = #doc_comment]
            #[derive(Clone, Debug)]
            pub struct #concept_id {
                #(#components)*
                #optional_ref
            }
        }
    };

    let optional_struct_def = if !optional_components.is_empty() {
        let doc_comment = format!("Optional part of [{}].", concept_id);

        let components = optional_components
            .iter()
            .map(|component| component.to_field_definition(true));

        Some(quote! {
            #[doc = #doc_comment]
            #[derive(Clone, Debug, Default)]
            pub struct #concept_optional_id {
                #(#components)*
            }
        })
    } else {
        None
    };

    let make = {
        let required = required_components.iter().map(|c| {
            let path = &c.path;
            let field_name = &c.id;

            quote! { with(#path(), self.#field_name) }
        });

        let optional = optional_components.iter().map(|c| {
            let path = &c.path;
            let field_name = &c.id;

            quote! {
                if let Some(#field_name) = self.optional.#field_name {
                    entity.set(#path(), #field_name);
                }
            }
        });

        quote! {
            fn make(self) -> Entity {
                let mut entity = Entity::new()
                    #(.#required)*;

                #(#optional)*

                entity
            }
        }
    };

    let get_spawned = {
        let required_components = required_components
            .iter()
            .map(|c| c.with_id_and_path(|f, p| quote! { #f: entity::get_component(id, #p())?, }));

        let optional = if optional_components.is_empty() {
            None
        } else {
            let optional_components = optional_components.iter().map(|c| {
                c.with_id_and_path(|f, p| quote! { #f: entity::get_component(id, #p()), })
            });

            Some(quote! {
                optional: #concept_optional_id {
                    #(#optional_components)*
                }
            })
        };

        quote! {
            fn get_spawned(id: EntityId) -> Option<Self> {
                Some(Self {
                    #(#required_components)*
                    #optional
                })
            }
        }
    };

    let get_unspawned = {
        let required_components = required_components
            .iter()
            .map(|c| c.with_id_and_path(|f, p| quote! { #f: entity.get(#p())?, }));

        let optional = if optional_components.is_empty() {
            None
        } else {
            let optional_components = optional_components
                .iter()
                .map(|c| c.with_id_and_path(|f, p| quote! { #f: entity.get(#p()), }));

            Some(quote! {
                optional: #concept_optional_id {
                    #(#optional_components)*
                }
            })
        };

        quote! {
            fn get_unspawned(entity: &Entity) -> Option<Self> {
                Some(Self {
                    #(#required_components)*
                    #optional
                })
            }
        }
    };

    let contained_by = {
        let required_paths = required_components
            .iter()
            .map(|c| &c.path)
            .collect::<Vec<_>>();

        quote! {
            fn contained_by_spawned(id: EntityId) -> bool {
                entity::has_components(id, &[
                    #(&#required_paths()),*
                ])
            }

            fn contained_by_unspawned(entity: &Entity) -> bool {
                entity.has_components(&[
                    #(&#required_paths()),*
                ])
            }
        }
    };

    let optional_field = if optional_components.is_empty() {
        None
    } else {
        Some(quote! { optional: Default::default(), })
    };

    let concept_suggested = if required_components.iter().all(|c| c.suggested.is_some()) {
        let required_components = required_components
            .iter()
            .map(|c| {
                let field_name = &c.id;
                let suggested = value_to_token_stream(items, c.suggested.unwrap())?;

                anyhow::Ok(quote! { #field_name: #suggested, })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Some(quote! {
            impl ConceptSuggested for #concept_id {
                fn suggested() -> Self {
                    Self {
                        #(#required_components)*
                        #optional_field
                    }
                }
            }
        })
    } else {
        None
    };

    let concept_components = {
        let (required_component_types, required_component_calls): (Vec<_>, Vec<_>) =
            required_components.iter().map(|c| (&c.ty, &c.path)).unzip();
        let (optional_component_types, optional_component_calls): (Vec<_>, Vec<_>) =
            optional_components.iter().map(|c| (&c.ty, &c.path)).unzip();

        let from_required_fields = required_components.iter().enumerate().map(|(i, c)| {
            let field_name = &c.id;
            let i = syn::Index::from(i);
            quote! {
                #field_name: required.#i,
            }
        });

        quote! {
            impl ConceptComponents for #concept_id {
                type Required = (#(Component<#required_component_types>,)*);
                type Optional = (#(Component<#optional_component_types>,)*);

                fn required() -> Self::Required {
                    (#(#required_component_calls(),)*)
                }

                fn optional() -> Self::Optional {
                    (#(#optional_component_calls(),)*)
                }

                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        #(#from_required_fields)*
                        #optional_field
                    }
                }
            }
        }
    };

    Ok(quote! {
        #struct_def
        #optional_struct_def
        impl Concept for #concept_id {
            #make
            #get_spawned
            #get_unspawned
            #contained_by
        }
        #concept_suggested
        #concept_components
    })
}

struct ComponentField<'a> {
    doc_comment: String,
    id: &'a Identifier,
    ty: TokenStream,
    path: TokenStream,
    suggested: Option<&'a Value>,
}
impl ComponentField<'_> {
    fn to_field_definition(&self, use_option: bool) -> TokenStream {
        let doc = &self.doc_comment;
        let id = self.id;
        let ty = &self.ty;

        let ty = if use_option {
            quote! { Option<#ty> }
        } else {
            ty.clone()
        };

        quote! {
            #[doc = #doc]
            pub #id: #ty,
        }
    }

    fn with_id_and_path(
        &self,
        f: impl Fn(&Identifier, &TokenStream) -> TokenStream,
    ) -> TokenStream {
        f(self.id, &self.path)
    }
}

fn component_to_field<'a>(
    items: &'a ItemMap,
    type_printer: &TypePrinter,
    context: Context,
    component_item_id: ItemId<Component>,
    value: &'a ConceptValue,
) -> anyhow::Result<ComponentField<'a>> {
    let component = items.get(component_item_id);
    let component_id = &component.data.id;

    let component_ty =
        type_printer.get(context, items, None, component.type_.as_resolved().unwrap())?;

    let mut doc_comment = String::new();

    writeln!(
        doc_comment,
        "**Component**: `{}`",
        items.fully_qualified_display_path(component, None, None)
    )?;
    writeln!(doc_comment)?;

    if let Some(value) = value.suggested.as_ref().and_then(|v| v.as_resolved()) {
        writeln!(
            doc_comment,
            "**Suggested value**: `{}`",
            SemiprettyTokenStream(value_to_token_stream(items, value)?)
        )?;
        writeln!(doc_comment)?;
    }

    if let Some(description) = &value.description {
        writeln!(doc_comment, "**Description**: {description}")?;
        writeln!(doc_comment)?;
    }

    if let Some(description) = &component.description {
        writeln!(doc_comment, "**Component description**: {}", description)?;
        writeln!(doc_comment)?;
    }

    let component_path = context.get_path(items, None, component_item_id)?;

    Ok(ComponentField {
        doc_comment,
        id: component_id,
        ty: component_ty,
        path: component_path,
        suggested: value.suggested.as_ref().and_then(|v| v.as_resolved()),
    })
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
