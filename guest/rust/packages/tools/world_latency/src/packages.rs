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
    pub mod fclqpkeyujrl3jeb6na6qmkl6jsumyoq {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("fclqpkeyujrl3jeb6na6qmkl6jsumyoq")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Auto-generated component definitions."]
        pub mod components {
            use ambient_api::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
                prelude::*,
            };
            static PLAYER_LAST_FRAME: Lazy<Component<u64>> = Lazy::new(|| {
                __internal_get_component("fclqpkeyujrl3jeb6na6qmkl6jsumyoq::player_last_frame")
            });
            #[doc = "**player_last_frame**: Last frame number reported by player\n\n*Attributes*: Debuggable, Networked"]
            pub fn player_last_frame() -> Component<u64> {
                *PLAYER_LAST_FRAME
            }
            static SERVER_FRAME: Lazy<Component<u64>> = Lazy::new(|| {
                __internal_get_component("fclqpkeyujrl3jeb6na6qmkl6jsumyoq::server_frame")
            });
            #[doc = "**server_frame**: Current server frame number\n\n*Attributes*: Debuggable, Networked"]
            pub fn server_frame() -> Component<u64> {
                *SERVER_FRAME
            }
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
            #[doc = "**FrameSeen**"]
            pub struct FrameSeen {
                pub frame: u64,
            }
            impl FrameSeen {
                #[allow(clippy::too_many_arguments)]
                pub fn new(frame: impl Into<u64>) -> Self {
                    Self {
                        frame: frame.into(),
                    }
                }
            }
            impl Message for FrameSeen {
                fn id() -> &'static str {
                    "fclqpkeyujrl3jeb6na6qmkl6jsumyoq::FrameSeen"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.frame.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        frame: u64::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for FrameSeen {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
}
pub use raw::fclqpkeyujrl3jeb6na6qmkl6jsumyoq as this;
