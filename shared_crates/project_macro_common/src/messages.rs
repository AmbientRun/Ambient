use ambient_project_semantic::{Item, ItemId, ItemMap, ItemSource, Scope};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{make_path, Context, TypePrinter};

pub fn make_definitions(
    context: Context,
    items: &ItemMap,
    type_printer: &TypePrinter,
    root_scope_id: ItemId<Scope>,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let messages = scope
        .messages
        .values()
        .filter_map(|m| context.extract_item_if_relevant(items, *m))
        .map(|message| {
            let id = message.data().id.as_str();

            let doc_comment = if let Some(desc) = &message.description {
                format!("**{}**: {}", id, desc)
            } else {
                format!("**{}**", id)
            };

            let struct_name = make_path(id);

            let fields = message.fields.iter().map(|f| {
                let name = make_path(f.0.as_str());
                let ty = type_printer
                    .get(
                        context,
                        items,
                        None,
                        root_scope_id,
                        f.1.as_resolved().unwrap(),
                    )
                    .unwrap();
                quote! { pub #name: #ty }
            });

            let new_parameters = message.fields.iter().map(|f| {
                let name = make_path(f.0.as_str());
                let ty = type_printer
                    .get(
                        context,
                        items,
                        None,
                        root_scope_id,
                        f.1.as_resolved().unwrap(),
                    )
                    .unwrap();
                quote! { #name: impl Into<#ty> }
            });

            let new_fields = message.fields.iter().map(|f| {
                let name = make_path(f.0.as_str());
                quote! { #name: #name.into() }
            });

            let serialize_fields = message.fields.iter().map(|f| {
                let name = make_path(f.0.as_str());
                quote! { self.#name.serialize_message_part(&mut output)? }
            });

            let deserialize_fields = message.fields.iter().map(|f| {
                let name = make_path(f.0.as_str());
                let ty = type_printer
                    .get(
                        context,
                        items,
                        None,
                        root_scope_id,
                        f.1.as_resolved().unwrap(),
                    )
                    .unwrap();
                quote! { #name: #ty ::deserialize_message_part(&mut input)? }
            });

            let message_impl = if message.data().source == ItemSource::Ambient {
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
                    #[allow(clippy::too_many_arguments)]
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

            Context::GuestApi | Context::GuestUser => {
                let api_path = context.guest_api_path().unwrap();
                quote! {
                    use #api_path::{prelude::*, message::{Message, MessageSerde, MessageSerdeError, RuntimeMessage, ModuleMessage}};
                    #(#messages)*
                }
            }
        }
    };

    Ok(quote! {
        #inner
    })
}
