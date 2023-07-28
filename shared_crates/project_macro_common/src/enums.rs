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
            let members = enumeration.members.iter().map(|(id, _)| make_path(id.as_str())).collect::<Vec<_>>();
            let enum_fields = enumeration.members.iter().map(|(id, comment)| {
                let name = make_path(id.as_str());
                quote! {
                    #[doc = #comment]
                    #name
                }
            });

            let ecs_prefix = context.guest_api_path()
                .map(|path| quote! { #path::ecs:: })
                .unwrap_or_else(|| quote! { crate:: });

            let main = quote! {
                #[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
                #[doc = #doc_comment]
                pub enum #enum_name {
                    #(#enum_fields,)*
                }

                impl #ecs_prefix EnumComponent for #enum_name {
                    fn to_u32(&self) -> u32 {
                        match self {
                            #(
                                Self::#members => #enum_name::#members as u32,
                            )*
                        }
                    }

                    fn from_u32(value: u32) -> Option<Self> {
                        #(
                            if value == #enum_name::#members as u32 {
                                return Some(Self::#members);
                            }
                        )*

                        None
                    }
                }
            };

            let supported_value = if let Some(guest_api_path) = context.guest_api_path() {
                quote! {
                    impl #guest_api_path::ecs::SupportedValue for #enum_name {
                        fn from_result(result: #guest_api_path::ecs::WitComponentValue) -> Option<Self> {
                            use #ecs_prefix EnumComponent;
                            u32::from_result(result).and_then(Self::from_u32)
                        }

                        fn into_result(self) -> #guest_api_path::ecs::WitComponentValue {
                            use #ecs_prefix EnumComponent;
                            self.to_u32().into_result()
                        }
                    }
                    impl #guest_api_path::ecs::SupportedValue for Vec<#enum_name> {
                        fn from_result(result: #guest_api_path::ecs::WitComponentValue) -> Option<Self> {
                            use #ecs_prefix EnumComponent;
                            (Vec :: <u32> :: from_result(result)).and_then(|v| {
                                v.into_iter().map(|v| #enum_name::from_u32(v)).collect::<Option<Vec<_>>>()
                            })
                        }

                        fn into_result(self) -> #guest_api_path::ecs::WitComponentValue {
                            use #ecs_prefix EnumComponent;
                            self.into_iter().map(|v| v.to_u32()).collect::<Vec<_>>().into_result()
                        }
                    }
                    impl #guest_api_path::ecs::SupportedValue for Option<#enum_name> {
                        fn from_result(result: #guest_api_path::ecs::WitComponentValue) -> Option<Self> {
                            use #ecs_prefix EnumComponent;
                            u32::from_result(result).map(|v| #enum_name ::from_u32(v))
                        }

                        fn into_result(self) -> #guest_api_path::ecs::WitComponentValue {
                            use #ecs_prefix EnumComponent;
                            self.map(|v| v.to_u32()).into_result()
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
