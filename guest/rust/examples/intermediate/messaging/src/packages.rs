#[allow(
    unused,
    clippy::unit_arg,
    clippy::let_and_return,
    clippy::approx_constant,
    clippy::unused_unit
)]
mod raw {
    pub mod ambient_core {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("ambient_core")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod of4w7yibjeuokeyypqxmtgqklc6vthln {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("of4w7yibjeuokeyypqxmtgqklc6vthln")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
        #[doc = r" and with other modules."]
        pub mod messages {
            use ambient_api::{
                message::{
                    Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage,
                },
                prelude::*,
            };
            #[derive(Clone, Debug)]
            #[doc = "**Hello**: Sent when a client joins the server, then sent back from the server"]
            pub struct Hello {
                pub text: String,
                pub source_reliable: bool,
            }
            impl Hello {
                #[allow(clippy::too_many_arguments)]
                pub fn new(text: impl Into<String>, source_reliable: impl Into<bool>) -> Self {
                    Self {
                        text: text.into(),
                        source_reliable: source_reliable.into(),
                    }
                }
            }
            impl Message for Hello {
                fn id() -> &'static str {
                    "of4w7yibjeuokeyypqxmtgqklc6vthln::Hello"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.text.serialize_message_part(&mut output)?;
                    self.source_reliable.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        text: String::deserialize_message_part(&mut input)?,
                        source_reliable: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Hello {}
            #[derive(Clone, Debug)]
            #[doc = "**HelloWithoutBody**: Sent just to say hi"]
            pub struct HelloWithoutBody;
            impl HelloWithoutBody {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for HelloWithoutBody {
                fn id() -> &'static str {
                    "of4w7yibjeuokeyypqxmtgqklc6vthln::HelloWithoutBody"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for HelloWithoutBody {}
            impl Default for HelloWithoutBody {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**Local**: Sent until it's acknowledged"]
            pub struct Local {
                pub text: String,
            }
            impl Local {
                #[allow(clippy::too_many_arguments)]
                pub fn new(text: impl Into<String>) -> Self {
                    Self { text: text.into() }
                }
            }
            impl Message for Local {
                fn id() -> &'static str {
                    "of4w7yibjeuokeyypqxmtgqklc6vthln::Local"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.text.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        text: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Local {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
}
pub use raw::of4w7yibjeuokeyypqxmtgqklc6vthln as this;
