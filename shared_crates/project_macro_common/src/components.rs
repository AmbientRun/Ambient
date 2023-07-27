use std::collections::HashMap;

use ambient_project_semantic::{ItemId, ItemMap, Scope, Type};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{make_path, Context};

pub fn make_init(
    context: Context,
    items: &ItemMap,
    root_scope_id: ItemId<Scope>,
    root_scope: &Scope,
) -> anyhow::Result<TokenStream> {
    Ok(match context {
        Context::Host => {
            let mut namespaces = vec![];
            root_scope.visit_recursive(items, |scope| {
                if !scope.components.is_empty() {
                    namespaces.push(syn::parse_str::<syn::Path>(
                        &items.fully_qualified_display_path_rust_style(
                            scope,
                            Some(root_scope_id),
                            None,
                        )?,
                    )?);
                }
                Ok(())
            })?;

            quote! {
                pub fn init() {
                    #(
                        #namespaces::components::init_components();
                    )*
                }
            }
        }
        Context::GuestApi { .. } | Context::GuestUser { .. } => quote! {},
    })
}

pub fn make_definitions(
    context: Context,
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, proc_macro2::TokenStream>,
    _root_scope_id: ItemId<Scope>,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let components = scope
        .components
        .values()
        .filter_map(|c| context.extract_item_if_relevant(items, *c))
        .map(|component| {
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
                Context::GuestApi | Context::GuestUser => {
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
                // lazy hack to get ambient_core components to work
                let namespace_path = namespace_path.strip_prefix("ambient_core::").unwrap();
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
            Context::GuestApi | Context::GuestUser => {
                let api_path = context.guest_api_path().unwrap();
                quote! {
                    use #api_path::{
                        once_cell::sync::Lazy, ecs::{Component, __internal_get_component},
                        prelude::*,
                    };
                    #(#components)*
                }
            }
        }
    };

    Ok(quote! {
        #inner
    })
}
