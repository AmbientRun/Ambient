use std::collections::HashMap;

use ambient_project_semantic::{Item, ItemId, ItemMap, Scope, Type};
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

    let messages = scope
        .messages
        .values()
        .map(|m| {
            let message = items.get(*m)?;
            let id = message.data().id.as_upper_camel_case();

            let doc_comment = if let Some(desc) = &message.description {
                format!("**{}**: {}", id, desc)
            } else {
                format!("**{}**", id)
            };

            let struct_name = make_path(&id);

            let fields = message.fields.iter().map(|f| {
                let name = f.0;
                let ty = &type_map[&f.1.as_resolved().unwrap()];
                quote! { pub #name: #ty }
            });

            let new_parameters = message.fields.iter().map(|f| {
                let name = f.0;
                let ty = &type_map[&f.1.as_resolved().unwrap()];
                quote! { #name: impl Into<#ty> }
            });

            let new_fields = message.fields.iter().map(|f| {
                let name = f.0;
                quote! { #name: #name.into() }
            });

            let serialize_fields = message.fields.iter().map(|f| {
                let name = f.0;
                quote! { self.#name.serialize_message_part(&mut output)? }
            });

            let deserialize_fields = message.fields.iter().map(|f| {
                let name = f.0;
                let ty = &type_map[&f.1.as_resolved().unwrap()];
                quote! { #name: #ty ::deserialize_message_part(&mut input)? }
            });

            let message_impl = if message.data().is_ambient_api {
                quote! { RuntimeMessage }
            } else {
                quote! { ModuleMessage }
            };

            Ok(quote! {
                #[derive(Clone, Debug)]
                #[doc = #doc_comment]
                pub struct #struct_name {
                    #(#fields,)*
                }
                impl #struct_name {
                    pub fn new(#(#new_parameters,)*) -> Self {
                        Self {
                            #(#new_fields,)*
                        }
                    }
                }
                impl Message for #struct_name {
                    fn id() -> &'static str {
                        #id
                    }
                    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                        let mut output = vec![];
                        #(#serialize_fields;)*
                        Ok(output)
                    }
                    fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                        Ok(Self {
                            #(#deserialize_fields,)*
                        })
                    }
                }
                impl #message_impl for #struct_name {}
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let inner = if messages.is_empty() {
        quote! {}
    } else {
        match context {
            Context::Host => quote! {
                use ambient_project_rt::message_serde::{Message, MessageSerde, MessageSerdeError, RuntimeMessage};
                use glam::{Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, Mat4, Quat};
                use crate::{EntityId, Entity};
                #(#messages)*
            },

            Context::Guest { api_path, .. } => quote! {
                use #api_path::{prelude::*, message::{Message, MessageSerde, MessageSerdeError, RuntimeMessage, ModuleMessage}};
                #(#messages)*
            },
        }
    };

    Ok(quote! {
        #(#scopes)*
        #inner
    })
}
