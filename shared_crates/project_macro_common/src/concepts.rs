use std::collections::HashMap;

use ambient_project_semantic::{ItemId, ItemMap, Scope, Type};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{make_path, Context};

pub fn make_definitions(
    context: &Context,
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, proc_macro2::TokenStream>,
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
        .map(|m| Ok(quote! {}))
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
