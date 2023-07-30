use ambient_project_semantic::{ItemId, ItemMap, Scope};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{make_path, Context, TypePrinter};

pub fn make_scopes(
    context: Context,
    items: &ItemMap,
    type_printer: &TypePrinter,
    root_scope_id: ItemId<Scope>,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let scopes = scope
        .scopes
        .values()
        .map(|s| {
            let scope = items.get(*s)?;
            let id = make_path(scope.data.id.as_str());
            let inner = make_scopes(context, items, type_printer, root_scope_id, &scope)?;
            if !inner.is_empty() {
                Ok(quote! {
                    #[allow(unused)]
                    pub mod #id {
                        #inner
                    }
                })
            } else {
                Ok(quote! {})
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let components = {
        let inner = crate::components::make_definitions(
            context,
            items,
            type_printer,
            root_scope_id,
            scope,
        )?;
        if inner.is_empty() {
            quote! {}
        } else {
            quote! {
                /// Auto-generated component definitions.
                pub mod components {
                    #inner
                }
            }
        }
    };
    let concepts = {
        let inner =
            crate::concepts::make_definitions(context, items, type_printer, root_scope_id, scope)?;
        if inner.is_empty() {
            quote! {}
        } else {
            quote! {
                /// Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept.
                ///
                /// They do not have any runtime representation outside of the components that compose them.
                pub mod concepts {
                    #inner
                }
            }
        }
    };
    let messages = {
        let inner =
            crate::messages::make_definitions(context, items, type_printer, root_scope_id, scope)?;
        if inner.is_empty() {
            quote! {}
        } else {
            quote! {
                /// Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,
                /// and with other modules.
                pub mod messages {
                    #inner
                }
            }
        }
    };
    let types = {
        let inner = crate::enums::make_definitions(context, items, scope)?;
        if inner.is_empty() {
            quote! {}
        } else {
            let includes = context
                .guest_api_path()
                .map(|s| quote! { use #s::{global::serde, message::*}; })
                .unwrap_or(quote! { use serde; use ambient_project_rt::message_serde::*; });
            quote! {
                /// Auto-generated type definitions.
                pub mod types {
                    #includes
                    #inner
                }
            }
        }
    };

    Ok(quote! {
        #(#scopes)*

        #components
        #concepts
        #messages
        #types
    })
}
