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
    pub mod ar7fnf3vl72bdb77nvnzbjpvps3lhvas {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("ar7fnf3vl72bdb77nvnzbjpvps3lhvas")
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
            static TRACK: Lazy<Component<u32>> =
                Lazy::new(|| __internal_get_component("ar7fnf3vl72bdb77nvnzbjpvps3lhvas::track"));
            #[doc = "**Track**: A track is a sequence of notes. The value corresponds to the index of the track.\n\n*Attributes*: Networked, Debuggable"]
            pub fn track() -> Component<u32> {
                *TRACK
            }
            static TRACK_AUDIO_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("ar7fnf3vl72bdb77nvnzbjpvps3lhvas::track_audio_url")
            });
            #[doc = "**Track Audio URL**: The URL of the audio file to play for a given track.\n\n*Attributes*: Networked, Debuggable"]
            pub fn track_audio_url() -> Component<String> {
                *TRACK_AUDIO_URL
            }
            static TRACK_NOTE_SELECTION: Lazy<Component<Vec<u32>>> = Lazy::new(|| {
                __internal_get_component("ar7fnf3vl72bdb77nvnzbjpvps3lhvas::track_note_selection")
            });
            #[doc = "**Track note selection**: The notes that are currently selected for a given track.\n\n*Attributes*: Networked, Debuggable"]
            pub fn track_note_selection() -> Component<Vec<u32>> {
                *TRACK_NOTE_SELECTION
            }
            static NEXT_PLAYER_HUE: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("ar7fnf3vl72bdb77nvnzbjpvps3lhvas::next_player_hue")
            });
            #[doc = "**Next Player Hue**: Controls the hue (in degrees) to use for the next player's color.\n\n*Attributes*: Debuggable, Resource"]
            pub fn next_player_hue() -> Component<u32> {
                *NEXT_PLAYER_HUE
            }
            static PLAYER_HUE: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("ar7fnf3vl72bdb77nvnzbjpvps3lhvas::player_hue")
            });
            #[doc = "**Player Hue**\n\n*Attributes*: Networked, Debuggable"]
            pub fn player_hue() -> Component<u32> {
                *PLAYER_HUE
            }
            static BPM: Lazy<Component<u32>> =
                Lazy::new(|| __internal_get_component("ar7fnf3vl72bdb77nvnzbjpvps3lhvas::bpm"));
            #[doc = "**BPM**: The number of beats per minute.\n\n*Attributes*: Networked, Debuggable"]
            pub fn bpm() -> Component<u32> {
                *BPM
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
            #[doc = "**Click**: Select or deselect a note."]
            pub struct Click {
                pub track_id: EntityId,
                pub index: u32,
            }
            impl Click {
                #[allow(clippy::too_many_arguments)]
                pub fn new(track_id: impl Into<EntityId>, index: impl Into<u32>) -> Self {
                    Self {
                        track_id: track_id.into(),
                        index: index.into(),
                    }
                }
            }
            impl Message for Click {
                fn id() -> &'static str {
                    "ar7fnf3vl72bdb77nvnzbjpvps3lhvas::Click"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.track_id.serialize_message_part(&mut output)?;
                    self.index.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        track_id: EntityId::deserialize_message_part(&mut input)?,
                        index: u32::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Click {}
            #[derive(Clone, Debug)]
            #[doc = "**SetBpm**: Set the BPM."]
            pub struct SetBpm {
                pub bpm: u32,
            }
            impl SetBpm {
                #[allow(clippy::too_many_arguments)]
                pub fn new(bpm: impl Into<u32>) -> Self {
                    Self { bpm: bpm.into() }
                }
            }
            impl Message for SetBpm {
                fn id() -> &'static str {
                    "ar7fnf3vl72bdb77nvnzbjpvps3lhvas::SetBpm"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.bpm.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        bpm: u32::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for SetBpm {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
}
pub use raw::ar7fnf3vl72bdb77nvnzbjpvps3lhvas as this;
