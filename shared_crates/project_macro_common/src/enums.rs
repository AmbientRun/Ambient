use ambient_project_semantic::{Item, ItemMap, Scope};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{make_path, Context};

pub fn make_definitions(
    context: Context,
    items: &ItemMap,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let enums = scope
        .types
        .values()
        .filter_map(|id| context.extract_item_if_relevant(items, *id))
        .filter(|ty| ty.inner.as_enum().is_some())
        .map(|ty| {
            let (data, enumeration) = (ty.data(), ty.inner.as_enum().unwrap());
            let id = data.id.as_str();
            let doc_comment = if let Some(desc) = &enumeration.description {
                format!("**{}**: {}", id, desc)
            } else {
                format!("**{}**", id)
            };

            let enum_name = make_path(id);

            let members = enumeration.members.iter().map(|(id, comment)| {
                let name = make_path(id.as_str());
                quote! {
                    #[doc = #comment]
                    #name
                }
            });

            let main = quote! {
                #[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
                #[doc = #doc_comment]
                pub enum #enum_name {
                    #(#members,)*
                }
            };

            let supported_value = if let Some(guest_api_path) = context.guest_api_path() {
                quote! {
                    impl #guest_api_path::ecs::SupportedValue for #enum_name {
                        fn from_result(result: #guest_api_path::ecs::WitComponentValue) -> Option<Self> {
                            unimplemented!()
                        }

                        fn into_result(self) -> #guest_api_path::ecs::WitComponentValue {
                            unimplemented!()
                        }
                    }
                    impl #guest_api_path::ecs::SupportedValue for Vec<#enum_name> {
                        fn from_result(result: #guest_api_path::ecs::WitComponentValue) -> Option<Self> {
                            unimplemented!()
                        }

                        fn into_result(self) -> #guest_api_path::ecs::WitComponentValue {
                            unimplemented!()
                        }
                    }
                    impl #guest_api_path::ecs::SupportedValue for Option<#enum_name> {
                        fn from_result(result: #guest_api_path::ecs::WitComponentValue) -> Option<Self> {
                            unimplemented!()
                        }

                        fn into_result(self) -> #guest_api_path::ecs::WitComponentValue {
                            unimplemented!()
                        }
                    }
                }
            } else {
                quote! {}
            };

            Ok(quote! {
                #main
                #supported_value
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let inner = if enums.is_empty() {
        quote! {}
    } else {
        match context {
            Context::Host => quote! {
                #(#enums)*
            },

            Context::GuestApi | Context::GuestUser => {
                quote! {
                    #(#enums)*
                }
            }
        }
    };

    Ok(quote! {
        #inner
    })
}
