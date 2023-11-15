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
    pub mod t6opuz533binrqqjsbgcezprtfa6vpyy {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("t6opuz533binrqqjsbgcezprtfa6vpyy")
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
            static PLAYER_MOVEMENT_DIRECTION: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component(
                    "t6opuz533binrqqjsbgcezprtfa6vpyy::player_movement_direction",
                )
            });
            #[doc = "**Player Movement Direction**: Direction of player movement"]
            pub fn player_movement_direction() -> Component<f32> {
                *PLAYER_MOVEMENT_DIRECTION
            }
            static TRACK_AUDIO_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("t6opuz533binrqqjsbgcezprtfa6vpyy::track_audio_url")
            });
            #[doc = "**Track Audio URL**: URL of the track audio\n\n*Attributes*: Networked, Debuggable"]
            pub fn track_audio_url() -> Component<String> {
                *TRACK_AUDIO_URL
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
            #[doc = "**Input**: Describes the input state of the player."]
            pub struct Input {
                pub direction: f32,
                pub start: bool,
            }
            impl Input {
                #[allow(clippy::too_many_arguments)]
                pub fn new(direction: impl Into<f32>, start: impl Into<bool>) -> Self {
                    Self {
                        direction: direction.into(),
                        start: start.into(),
                    }
                }
            }
            impl Message for Input {
                fn id() -> &'static str {
                    "t6opuz533binrqqjsbgcezprtfa6vpyy::Input"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.direction.serialize_message_part(&mut output)?;
                    self.start.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        direction: f32::deserialize_message_part(&mut input)?,
                        start: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Input {}
            #[derive(Clone, Debug)]
            #[doc = "**Ping**: Time to ping sound."]
            pub struct Ping;
            impl Ping {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for Ping {
                fn id() -> &'static str {
                    "t6opuz533binrqqjsbgcezprtfa6vpyy::Ping"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for Ping {}
            impl Default for Ping {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
}
pub use raw::t6opuz533binrqqjsbgcezprtfa6vpyy as this;
