#[allow(
    unused,
    clippy::unit_arg,
    clippy::let_and_return,
    clippy::approx_constant,
    clippy::unused_unit
)]
mod raw {
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
    pub mod tijz7x6fimbgu24sbbtp4nllhfxbgblp {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("tijz7x6fimbgu24sbbtp4nllhfxbgblp")
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
            static IS_ORBIT_CAMERA: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("tijz7x6fimbgu24sbbtp4nllhfxbgblp::is_orbit_camera")
            });
            #[doc = "**is_orbit_camera**\n\n*Attributes*: Debuggable, Networked"]
            pub fn is_orbit_camera() -> Component<()> {
                *IS_ORBIT_CAMERA
            }
            static CAMERA_ANGLE: Lazy<Component<Vec2>> = Lazy::new(|| {
                __internal_get_component("tijz7x6fimbgu24sbbtp4nllhfxbgblp::camera_angle")
            });
            #[doc = "**camera_angle**: Camera angle specified in radians; x = yaw, y = pitch\n\n*Attributes*: Debuggable, Networked"]
            pub fn camera_angle() -> Component<Vec2> {
                *CAMERA_ANGLE
            }
            static CAMERA_DISTANCE: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("tijz7x6fimbgu24sbbtp4nllhfxbgblp::camera_distance")
            });
            #[doc = "**camera_distance**: Camera distance specified in meters\n\n*Attributes*: Debuggable, Networked"]
            pub fn camera_distance() -> Component<f32> {
                *CAMERA_DISTANCE
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        #[doc = r""]
        #[doc = r" They do not have any runtime representation outside of the components that compose them."]
        pub mod concepts {
            use ambient_api::{
                global::serde::{self, Deserialize, Serialize},
                prelude::*,
            };
            #[doc = "**Orbit Camera**: An orbit camera.\n\n**Required**:\n- `is_orbit_camera`: No description provided.\n\n\n**Optional**:\n- `camera_angle`: Camera angle specified in radians; x = yaw, y = pitch\n- `camera_distance`: Camera distance specified in meters\n- `lookat_target`: The position that this entity should be looking at."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct OrbitCamera {
                #[doc = "**Component**: `tijz7x6fimbgu24sbbtp4nllhfxbgblp::is_orbit_camera`\n\n**Suggested value**: `()`\n\n"]
                pub is_orbit_camera: (),
                #[doc = r" Optional components."]
                pub optional: OrbitCameraOptional,
            }
            #[doc = "Optional part of [OrbitCamera]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct OrbitCameraOptional {
                #[doc = "**Component**: `tijz7x6fimbgu24sbbtp4nllhfxbgblp::camera_angle`\n\n**Component description**: Camera angle specified in radians; x = yaw, y = pitch\n\n"]
                pub camera_angle: Option<Vec2>,
                #[doc = "**Component**: `tijz7x6fimbgu24sbbtp4nllhfxbgblp::camera_distance`\n\n**Component description**: Camera distance specified in meters\n\n"]
                pub camera_distance: Option<f32>,
                #[doc = "**Component**: `ambient_core::transform::lookat_target`\n\n**Component description**: The position that this entity should be looking at.\n\n"]
                pub lookat_target: Option<Vec3>,
            }
            impl Concept for OrbitCamera {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera () , self . is_orbit_camera) ;
                    if let Some(camera_angle) = self.optional.camera_angle {
                        entity . set (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_angle () , camera_angle) ;
                    }
                    if let Some(camera_distance) = self.optional.camera_distance {
                        entity . set (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_distance () , camera_distance) ;
                    }
                    if let Some(lookat_target) = self.optional.lookat_target {
                        entity.set(
                            ambient_api::core::transform::components::lookat_target(),
                            lookat_target,
                        );
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { is_orbit_camera : entity :: get_component (id , crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera ()) ? , optional : OrbitCameraOptional { camera_angle : entity :: get_component (id , crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_angle ()) , camera_distance : entity :: get_component (id , crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_distance ()) , lookat_target : entity :: get_component (id , ambient_api :: core :: transform :: components :: lookat_target ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { is_orbit_camera : entity . get (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera ()) ? , optional : OrbitCameraOptional { camera_angle : entity . get (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_angle ()) , camera_distance : entity . get (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_distance ()) , lookat_target : entity . get (ambient_api :: core :: transform :: components :: lookat_target ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera ()])
                }
            }
            impl ConceptSuggested for OrbitCamera {
                #[doc = "```\nis_orbit_camera: (),\n```"]
                fn suggested() -> Self {
                    Self {
                        is_orbit_camera: (),
                        optional: Default::default(),
                    }
                }
            }
            impl ConceptComponents for OrbitCamera {
                type Required = (Component<()>,);
                type Optional = (Component<Vec2>, Component<f32>, Component<Vec3>);
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera () ,)
                }
                fn optional() -> Self::Optional {
                    (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_angle () , crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_distance () , ambient_api :: core :: transform :: components :: lookat_target () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        is_orbit_camera: required.0,
                        optional: Default::default(),
                    }
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
    pub mod xjoy27d3i4hc4l7fmlm5pacc2cmxlnlw {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("xjoy27d3i4hc4l7fmlm5pacc2cmxlnlw")
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
pub use raw::t33j53muycmj4i66en5lheneowad5hbz as deps_code;
pub use raw::tijz7x6fimbgu24sbbtp4nllhfxbgblp as orbit_camera;
pub use raw::viyiawgsl5lsiul6pup6pyv6bbt6o3vw as deps_assets;
pub use raw::xjoy27d3i4hc4l7fmlm5pacc2cmxlnlw as this;
