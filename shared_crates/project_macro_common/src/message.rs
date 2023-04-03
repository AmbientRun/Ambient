use super::{
    component::type_to_token_stream,
    tree::{Tree, TreeNode},
    util, Context,
};
use ambient_project::Message;
use proc_macro2::{Span, TokenStream};
use quote::quote;

pub fn tree_to_token_stream(
    message_tree: &Tree<Message>,
    context: &Context,
) -> anyhow::Result<TokenStream> {
    to_token_stream(
        message_tree.root(),
        context,
        |context, _ns, ts| match context {
            Context::Host => quote! {
                use ambient_project_rt::message_serde::{Message, MessageSerde, MessageSerdeError};
                use glam::{Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, Mat4, Quat};
                use crate::{EntityId, Entity};
                #ts
            },
            Context::Guest { api_path, .. } => quote! {
                use #api_path::{prelude::*, message::{Message, MessageSerde, MessageSerdeError}};
                #ts
            },
        },
    )
}

fn to_token_stream(
    node: &TreeNode<Message>,
    context: &Context,
    wrapper: impl Fn(&Context, &TreeNode<Message>, TokenStream) -> TokenStream + Copy,
) -> anyhow::Result<TokenStream> {
    util::tree_to_token_stream(
        node,
        context,
        wrapper,
        to_token_stream,
        |id, message, context| {
            let doc_comment = format!("**{}**: {}", message.name, message.description);

            let struct_name = syn::Ident::new(
                &id.split('_')
                    .map(|segment| {
                        let mut c = segment.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<String>(),
                Span::call_site(),
            );

            let fields = message
                .fields
                .iter()
                .map(|f| {
                    let name = f.0;
                    type_to_token_stream(f.1, context, false).map(|ty| {
                        quote! { pub #name: #ty }
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let new_parameters = message
                .fields
                .iter()
                .map(|f| {
                    let name = f.0;
                    type_to_token_stream(f.1, context, false).map(|ty| {
                        quote! { #name: impl Into<#ty> }
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let new_fields = message.fields.iter().map(|f| {
                let name = f.0;
                quote! { #name: #name.into() }
            });

            let serialize_fields = message.fields.iter().map(|f| {
                let name = f.0;
                quote! { self.#name.serialize_message_part(&mut output)? }
            });

            let deserialize_fields = message
                .fields
                .iter()
                .map(|f| {
                    let name = f.0;
                    type_to_token_stream(f.1, context, true).map(|ty| {
                        quote! { #name: #ty ::deserialize_message_part(&mut input)? }
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

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
            })
        },
    )
}
