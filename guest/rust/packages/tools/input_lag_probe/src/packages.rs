#[allow(
    unused,
    clippy::unit_arg,
    clippy::let_and_return,
    clippy::approx_constant,
    clippy::unused_unit
)]
mod raw {
    pub mod inputlagprobeaghgqno42orb2j3abay {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("inputlagprobeaghgqno42orb2j3abay")
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
            static INPUT_TIMESTAMP: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component("inputlagprobeaghgqno42orb2j3abay::input_timestamp")
            });
            #[doc = "**input_timestamp**: Timestamp from the last Input message received from the player\n\n*Attributes*: Networked"]
            pub fn input_timestamp() -> Component<Duration> {
                *INPUT_TIMESTAMP
            }
            static INPUT_LAG: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component("inputlagprobeaghgqno42orb2j3abay::input_lag")
            });
            #[doc = "**input_lag**: Lag from the last Input message received from the player\n\n*Attributes*: Networked"]
            pub fn input_lag() -> Component<Duration> {
                *INPUT_LAG
            }
            static INPUT_FREQUENCY: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component("inputlagprobeaghgqno42orb2j3abay::input_frequency")
            });
            #[doc = "**input_frequency**: How frequently should the input messages be sent\n\n*Attributes*: Resource"]
            pub fn input_frequency() -> Component<Duration> {
                *INPUT_FREQUENCY
            }
            static SMOOTHING_FACTOR: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("inputlagprobeaghgqno42orb2j3abay::smoothing_factor")
            });
            #[doc = "**smoothing_factor**: How much smoothed value is affected by the current measurement (lower = more, 1 = just use the current value)\n\n*Attributes*: Resource"]
            pub fn smoothing_factor() -> Component<u32> {
                *SMOOTHING_FACTOR
            }
            static LAST_PROCESSED_TIMESTAMP: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component(
                    "inputlagprobeaghgqno42orb2j3abay::last_processed_timestamp",
                )
            });
            #[doc = "**last_processed_timestamp**: Last input_timestamp that was processed by the client\n\n*Attributes*: Resource"]
            pub fn last_processed_timestamp() -> Component<Duration> {
                *LAST_PROCESSED_TIMESTAMP
            }
            static LOCAL_LAG: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component("inputlagprobeaghgqno42orb2j3abay::local_lag")
            });
            #[doc = "**local_lag**: Current smoothed value of locally perceived input lag\n\n*Attributes*: Resource"]
            pub fn local_lag() -> Component<Duration> {
                *LOCAL_LAG
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
            #[doc = "**Input**"]
            pub struct Input {
                pub timestamp: Duration,
                pub lag: Duration,
            }
            impl Input {
                #[allow(clippy::too_many_arguments)]
                pub fn new(timestamp: impl Into<Duration>, lag: impl Into<Duration>) -> Self {
                    Self {
                        timestamp: timestamp.into(),
                        lag: lag.into(),
                    }
                }
            }
            impl Message for Input {
                fn id() -> &'static str {
                    "inputlagprobeaghgqno42orb2j3abay::Input"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.timestamp.serialize_message_part(&mut output)?;
                    self.lag.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        timestamp: Duration::deserialize_message_part(&mut input)?,
                        lag: Duration::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Input {}
            #[derive(Clone, Debug)]
            #[doc = "**ShowInputLagWindow**: Show input lag UI"]
            pub struct ShowInputLagWindow;
            impl ShowInputLagWindow {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for ShowInputLagWindow {
                fn id() -> &'static str {
                    "inputlagprobeaghgqno42orb2j3abay::ShowInputLagWindow"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for ShowInputLagWindow {}
            impl Default for ShowInputLagWindow {
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
}
pub use raw::inputlagprobeaghgqno42orb2j3abay as this;
