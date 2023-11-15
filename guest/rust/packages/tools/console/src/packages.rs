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
    pub mod wqkvthxjifxdwgi4hc4efmtjct6tljbw {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("wqkvthxjifxdwgi4hc4efmtjct6tljbw")
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
            #[doc = "**ConsoleServerInput**"]
            pub struct ConsoleServerInput {
                pub input: String,
            }
            impl ConsoleServerInput {
                #[allow(clippy::too_many_arguments)]
                pub fn new(input: impl Into<String>) -> Self {
                    Self {
                        input: input.into(),
                    }
                }
            }
            impl Message for ConsoleServerInput {
                fn id() -> &'static str {
                    "wqkvthxjifxdwgi4hc4efmtjct6tljbw::ConsoleServerInput"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.input.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        input: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for ConsoleServerInput {}
            #[derive(Clone, Debug)]
            #[doc = "**ConsoleServerOutput**"]
            pub struct ConsoleServerOutput {
                pub text: String,
                pub ty: u8,
                pub is_server: bool,
            }
            impl ConsoleServerOutput {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    text: impl Into<String>,
                    ty: impl Into<u8>,
                    is_server: impl Into<bool>,
                ) -> Self {
                    Self {
                        text: text.into(),
                        ty: ty.into(),
                        is_server: is_server.into(),
                    }
                }
            }
            impl Message for ConsoleServerOutput {
                fn id() -> &'static str {
                    "wqkvthxjifxdwgi4hc4efmtjct6tljbw::ConsoleServerOutput"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.text.serialize_message_part(&mut output)?;
                    self.ty.serialize_message_part(&mut output)?;
                    self.is_server.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        text: String::deserialize_message_part(&mut input)?,
                        ty: u8::deserialize_message_part(&mut input)?,
                        is_server: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for ConsoleServerOutput {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
}
pub use raw::wqkvthxjifxdwgi4hc4efmtjct6tljbw as this;
