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
    pub mod t33j53muycmj4i66en5lheneowad5hbz {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("t33j53muycmj4i66en5lheneowad5hbz")
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
            static SPAWNED_BY_US: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("t33j53muycmj4i66en5lheneowad5hbz::spawned_by_us")
            });
            #[doc = "**spawned_by_us**"]
            pub fn spawned_by_us() -> Component<()> {
                *SPAWNED_BY_US
            }
            static SPIN_DIRECTION: Lazy<
                Component<
                    crate::packages::raw::viyiawgsl5lsiul6pup6pyv6bbt6o3vw::types::SpinDirection,
                >,
            > = Lazy::new(|| {
                __internal_get_component("t33j53muycmj4i66en5lheneowad5hbz::spin_direction")
            });
            #[doc = "**spin_direction**\n\n*Attributes*: Enum"]
            pub fn spin_direction() -> Component<
                crate::packages::raw::viyiawgsl5lsiul6pup6pyv6bbt6o3vw::types::SpinDirection,
            > {
                *SPIN_DIRECTION
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
            #[doc = "**Spawn**: Spawn the asset."]
            pub struct Spawn {
                pub spin_speed: f32,
                pub spin_direction:
                    crate::packages::raw::viyiawgsl5lsiul6pup6pyv6bbt6o3vw::types::SpinDirection,
            }
            impl Spawn {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    spin_speed: impl Into<f32>,
                    spin_direction : impl Into < crate :: packages :: raw :: viyiawgsl5lsiul6pup6pyv6bbt6o3vw :: types :: SpinDirection >,
                ) -> Self {
                    Self {
                        spin_speed: spin_speed.into(),
                        spin_direction: spin_direction.into(),
                    }
                }
            }
            impl Message for Spawn {
                fn id() -> &'static str {
                    "t33j53muycmj4i66en5lheneowad5hbz::Spawn"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.spin_speed.serialize_message_part(&mut output)?;
                    self.spin_direction.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok (Self { spin_speed : f32 :: deserialize_message_part (& mut input) ? , spin_direction : crate :: packages :: raw :: viyiawgsl5lsiul6pup6pyv6bbt6o3vw :: types :: SpinDirection :: deserialize_message_part (& mut input) ? , })
                }
            }
            impl ModuleMessage for Spawn {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod viyiawgsl5lsiul6pup6pyv6bbt6o3vw {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("viyiawgsl5lsiul6pup6pyv6bbt6o3vw")
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
            static SPIN_SPEED: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("viyiawgsl5lsiul6pup6pyv6bbt6o3vw::spin_speed")
            });
            #[doc = "**Spin speed**"]
            pub fn spin_speed() -> Component<f32> {
                *SPIN_SPEED
            }
        }
        #[doc = r" Auto-generated type definitions."]
        pub mod types {
            use ambient_api::{global::serde, message::*};
            #[derive(
                Copy, Clone, Debug, PartialEq, Eq, serde :: Serialize, serde :: Deserialize, Default,
            )]
            #[serde(crate = "self::serde")]
            #[doc = "**SpinDirection**: The direction the cube should spin"]
            pub enum SpinDirection {
                #[default]
                #[doc = "Forward"]
                Forward,
                #[doc = "Backward"]
                Backward,
            }
            impl ambient_api::ecs::EnumComponent for SpinDirection {
                fn to_u32(&self) -> u32 {
                    match self {
                        Self::Forward => SpinDirection::Forward as u32,
                        Self::Backward => SpinDirection::Backward as u32,
                    }
                }
                fn from_u32(value: u32) -> Option<Self> {
                    if value == SpinDirection::Forward as u32 {
                        return Some(Self::Forward);
                    }
                    if value == SpinDirection::Backward as u32 {
                        return Some(Self::Backward);
                    }
                    None
                }
            }
            impl ambient_api::ecs::SupportedValue for SpinDirection {
                fn from_result(result: ambient_api::ecs::WitComponentValue) -> Option<Self> {
                    use ambient_api::ecs::EnumComponent;
                    u32::from_result(result).and_then(Self::from_u32)
                }
                fn into_result(self) -> ambient_api::ecs::WitComponentValue {
                    use ambient_api::ecs::EnumComponent;
                    self.to_u32().into_result()
                }
                fn from_value(value: ambient_api::ecs::ComponentValue) -> Option<Self> {
                    use ambient_api::ecs::EnumComponent;
                    u32::from_value(value).and_then(Self::from_u32)
                }
                fn into_value(self) -> ambient_api::ecs::ComponentValue {
                    use ambient_api::ecs::EnumComponent;
                    self.to_u32().into_value()
                }
            }
            impl MessageSerde for SpinDirection {
                fn serialize_message_part(
                    &self,
                    output: &mut Vec<u8>,
                ) -> Result<(), MessageSerdeError> {
                    ambient_api::ecs::EnumComponent::to_u32(self).serialize_message_part(output)
                }
                fn deserialize_message_part(
                    input: &mut dyn std::io::Read,
                ) -> Result<Self, MessageSerdeError> {
                    ambient_api::ecs::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                        .ok_or(MessageSerdeError::InvalidValue)
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
pub use raw::t33j53muycmj4i66en5lheneowad5hbz as this;
pub use raw::viyiawgsl5lsiul6pup6pyv6bbt6o3vw as ambient_example_deps_assets;
