use ambient_project_semantic::{Item, ItemMap, Scope};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{make_path, Context};

pub fn generate(context: Context, items: &ItemMap, scope: &Scope) -> anyhow::Result<TokenStream> {
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
                #[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
                #[serde(crate = "self::serde")]
                #[doc = #doc_comment]
                pub enum #enum_name {
                    #[default]
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
                    // Code generation for Vec/Option is disabled as you cannot implement foreign traits on foreign types
                    // Need to think about how to best solve this
                    // impl #guest_api_path::ecs::SupportedValue for Vec<#enum_name> {
                    //     fn from_result(result: #guest_api_path::ecs::WitComponentValue) -> Option<Self> {
                    //         use #ecs_prefix EnumComponent;
                    //         (Vec :: <u32> :: from_result(result)).and_then(|v| {
                    //             v.into_iter().map(|v| #enum_name::from_u32(v)).collect::<Option<Vec<_>>>()
                    //         })
                    //     }

                    //     fn into_result(self) -> #guest_api_path::ecs::WitComponentValue {
                    //         use #ecs_prefix EnumComponent;
                    //         self.into_iter().map(|v| v.to_u32()).collect::<Vec<_>>().into_result()
                    //     }
                    // }
                    // impl #guest_api_path::ecs::SupportedValue for Option<#enum_name> {
                    //     fn from_result(result: #guest_api_path::ecs::WitComponentValue) -> Option<Self> {
                    //         use #ecs_prefix EnumComponent;
                    //         u32::from_result(result).map(|v| #enum_name ::from_u32(v))
                    //     }

                    //     fn into_result(self) -> #guest_api_path::ecs::WitComponentValue {
                    //         use #ecs_prefix EnumComponent;
                    //         self.map(|v| v.to_u32()).into_result()
                    //     }
                    // }
                }
            } else {
                quote! {}
            };

            let message_serde_impl = quote! {
                impl MessageSerde for #enum_name {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        #ecs_prefix EnumComponent::to_u32(self).serialize_message_part(output)
                    }

                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        #ecs_prefix EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
            };

            Ok(quote! {
                #main
                #supported_value
                #message_serde_impl
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    if enums.is_empty() {
        return Ok(quote! {});
    }

    let includes = context
        .guest_api_path()
        .map(|s| quote! { use #s::{global::serde, message::*}; })
        .unwrap_or(quote! { use serde; use ambient_project_rt::message_serde::*; });

    Ok(quote! {
        /// Auto-generated type definitions.
        pub mod types {
            #includes
            #(#enums)*
        }
    })
}
