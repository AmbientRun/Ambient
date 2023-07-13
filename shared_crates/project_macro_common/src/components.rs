use std::collections::HashMap;

use ambient_project_semantic::{ItemId, ItemMap, Scope, Type};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{make_path, Context};

pub fn make_definitions(
    context: &Context,
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, proc_macro2::TokenStream>,
    root_scope_id: ItemId<Scope>,
    root_scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let inner = make_definitions_inner(context, items, type_map, root_scope_id, root_scope)?;
    let init = match context {
        Context::Host => {
            let mut namespaces = vec![];
            root_scope.visit_recursive(items, |scope| {
                if !scope.components.is_empty() {
                    namespaces.push(syn::parse_str::<syn::Path>(
                        &items
                            .fully_qualified_display_path_rust_style(scope, Some(root_scope_id))?,
                    )?);
                }
                Ok(())
            })?;

            quote! {
                pub fn init() {
                    #(
                        #namespaces::init_components();
                    )*
                }
            }
        }
        Context::Guest { .. } => quote! {},
    };

    Ok(quote! {
        #inner
        #init
    })
}

fn make_definitions_inner(
    context: &Context,
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, proc_macro2::TokenStream>,
    root_scope_id: ItemId<Scope>,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let scopes = scope
        .scopes
        .values()
        .map(|s| {
            let scope = items.get(*s)?;
            let id = make_path(&scope.data.id.as_snake_case());

            let inner = make_definitions_inner(context, items, type_map, root_scope_id, &scope)?;
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

    let components = scope
        .components
        .values()
        .map(|c| {
            let component = items.get(*c)?;
            let id = component.data.id.as_snake_case();
            let type_id = component.type_.as_resolved().expect("type was unresolved");
            let ty = type_map.get(&type_id).unwrap_or_else(|| {
                panic!(
                    "type not found: {}",
                    items
                        .fully_qualified_display_path_ambient_style(
                            &*items.get(type_id).unwrap(),
                            true,
                            None
                        )
                        .unwrap()
                )
            });

            let attributes: Vec<_> = component
                .attributes
                .iter()
                .filter_map(|id| id.as_resolved())
                .map(|id| items.get(id).unwrap().data.id.clone())
                .collect();

            let name = component
                .name
                .as_ref()
                .map(|x| x as &str)
                .unwrap_or_else(|| component.data.id.as_ref());
            let mut doc_comment = format!("**{}**", name);

            if let Some(desc) = &component.description {
                if !desc.is_empty() {
                    doc_comment += &format!(": {}", desc.replace('\n', "\n\n"));
                }
            }

            // Metadata
            if !component.attributes.is_empty() {
                let attributes: Vec<_> = attributes.iter().map(|id| id.to_string()).collect();
                doc_comment += &format!("\n\n*Attributes*: {}", attributes.join(", "))
            }
            if let Some(default) = component.default.as_ref().and_then(|c| c.as_resolved()) {
                doc_comment += &format!("\n\n*Suggested Default*: {default}")
            }

            let doc_comment = doc_comment.trim();

            match context {
                Context::Host => {
                    let ident = make_path(&id);
                    let attributes: Vec<_> = attributes
                        .into_iter()
                        .map(|s| make_path(&s.as_upper_camel_case()))
                        .collect();
                    let description = component.description.to_owned().unwrap_or_default();

                    Ok(quote! {
                        #[doc = #doc_comment]
                        @[#(#attributes,)* Name[#name], Description[#description]]
                        #ident: #ty,
                    })
                }
                Context::Guest { .. } => {
                    let component_id = items.fully_qualified_display_path_ambient_style(
                        &*component,
                        false,
                        None,
                    )?;
                    let ident = make_path(&id);
                    let uppercase_ident = make_path(&id.to_uppercase());

                    let component_init = quote! {
                        Lazy::new(|| __internal_get_component(#component_id))
                    };

                    Ok(quote! {
                        static #uppercase_ident: Lazy< Component< #ty > > = #component_init;

                        #[doc = #doc_comment]
                        pub fn #ident() -> Component<#ty> {
                            *#uppercase_ident
                        }
                    })
                }
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let inner = if components.is_empty() {
        quote! {}
    } else {
        match context {
            Context::Host => {
                let namespace_path =
                    items.fully_qualified_display_path_ambient_style(scope, false, None)?;
                // lazy hack to get ambient/core components to work
                let namespace_path = namespace_path.strip_prefix("ambient/core/").unwrap();
                quote! {
                    use std::time::Duration;
                    use glam::{Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, Mat4, Quat};
                    use crate::{EntityId, Debuggable, Networked, Store, Resource, MaybeResource, Name, Description, components};
                    use ambient_shared_types::{ProceduralMeshHandle, ProceduralTextureHandle, ProceduralSamplerHandle, ProceduralMaterialHandle};
                    components!(#namespace_path, {
                        #(#components)*
                    });
                }
            }
            Context::Guest {
                api_path,
                fully_qualified_path,
            } => {
                let fully_qualified_prefix = if *fully_qualified_path {
                    quote! { #api_path::global }
                } else {
                    quote! {}
                };
                quote! {
                    use #api_path::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    use #fully_qualified_prefix::{
                        EntityId, Mat4, Quat, Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, IVec2, IVec3, IVec4,
                        Duration, ProceduralMeshHandle, ProceduralTextureHandle, ProceduralSamplerHandle,
                        ProceduralMaterialHandle
                    };
                    #(#components)*
                }
            }
        }
    };

    Ok(quote! {
        #(#scopes)*
        #inner
    })
}
