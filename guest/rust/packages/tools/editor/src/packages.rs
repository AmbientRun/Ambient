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
    pub mod n7xfnlfzdmnvj7bqasfdhqftbtdi27ah {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("n7xfnlfzdmnvj7bqasfdhqftbtdi27ah")
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
            static IN_EDITOR: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("n7xfnlfzdmnvj7bqasfdhqftbtdi27ah::in_editor")
            });
            #[doc = "**in_editor**\n\n*Attributes*: Networked, Debuggable"]
            pub fn in_editor() -> Component<bool> {
                *IN_EDITOR
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
            #[doc = "**EditorLoad**: Sent by the editor when it has loaded to signal to other modules to add themselves."]
            pub struct EditorLoad;
            impl EditorLoad {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for EditorLoad {
                fn id() -> &'static str {
                    "n7xfnlfzdmnvj7bqasfdhqftbtdi27ah::EditorLoad"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for EditorLoad {}
            impl Default for EditorLoad {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**EditorMenuBarAdd**: Sent by other modules to add themselves to the editor's menubar."]
            pub struct EditorMenuBarAdd {
                pub name: String,
            }
            impl EditorMenuBarAdd {
                #[allow(clippy::too_many_arguments)]
                pub fn new(name: impl Into<String>) -> Self {
                    Self { name: name.into() }
                }
            }
            impl Message for EditorMenuBarAdd {
                fn id() -> &'static str {
                    "n7xfnlfzdmnvj7bqasfdhqftbtdi27ah::EditorMenuBarAdd"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.name.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        name: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for EditorMenuBarAdd {}
            #[derive(Clone, Debug)]
            #[doc = "**EditorMenuBarClick**: Sent by the editor when a menu item is clicked."]
            pub struct EditorMenuBarClick {
                pub name: String,
            }
            impl EditorMenuBarClick {
                #[allow(clippy::too_many_arguments)]
                pub fn new(name: impl Into<String>) -> Self {
                    Self { name: name.into() }
                }
            }
            impl Message for EditorMenuBarClick {
                fn id() -> &'static str {
                    "n7xfnlfzdmnvj7bqasfdhqftbtdi27ah::EditorMenuBarClick"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.name.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        name: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for EditorMenuBarClick {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod xar372tfo2oswb4pkvx7h7o3rxi6tap6 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("xar372tfo2oswb4pkvx7h7o3rxi6tap6")
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
    pub mod xr6whcy65gn3vlzrp2ssyn7udcbxb6mu {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu")
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
            static EDITOR_CAMERA: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::editor_camera")
            });
            #[doc = "**editor_camera**\n\n*Attributes*: Networked, Debuggable"]
            pub fn editor_camera() -> Component<EntityId> {
                *EDITOR_CAMERA
            }
            static MOUSEOVER_POSITION: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::mouseover_position")
            });
            #[doc = "**mouseover_position**\n\n*Attributes*: Networked, Debuggable"]
            pub fn mouseover_position() -> Component<Vec3> {
                *MOUSEOVER_POSITION
            }
            static MOUSEOVER_ENTITY: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::mouseover_entity")
            });
            #[doc = "**mouseover_entity**\n\n*Attributes*: Networked, Debuggable"]
            pub fn mouseover_entity() -> Component<EntityId> {
                *MOUSEOVER_ENTITY
            }
            static SELECTED_ENTITY: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::selected_entity")
            });
            #[doc = "**selected_entity**\n\n*Attributes*: Networked, Debuggable"]
            pub fn selected_entity() -> Component<EntityId> {
                *SELECTED_ENTITY
            }
            static HAS_SAMPLE_SCENE: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::has_sample_scene")
            });
            #[doc = "**has_sample_scene**\n\n*Attributes*: Networked, MaybeResource"]
            pub fn has_sample_scene() -> Component<()> {
                *HAS_SAMPLE_SCENE
            }
            static CAMERA_ANGLE: Lazy<Component<Vec2>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::camera_angle")
            });
            #[doc = "**camera_angle**: X is yaw, Y is pitch\n\n*Attributes*: Networked, Debuggable"]
            pub fn camera_angle() -> Component<Vec2> {
                *CAMERA_ANGLE
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
            #[doc = "**ToggleEditor**"]
            pub struct ToggleEditor {
                pub camera_transform: Option<Mat4>,
            }
            impl ToggleEditor {
                #[allow(clippy::too_many_arguments)]
                pub fn new(camera_transform: impl Into<Option<Mat4>>) -> Self {
                    Self {
                        camera_transform: camera_transform.into(),
                    }
                }
            }
            impl Message for ToggleEditor {
                fn id() -> &'static str {
                    "xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::ToggleEditor"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.camera_transform.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        camera_transform: Option::<Mat4>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for ToggleEditor {}
            #[derive(Clone, Debug)]
            #[doc = "**Input**"]
            pub struct Input {
                pub aim_delta: Vec2,
                pub movement: Vec2,
                pub boost: bool,
                pub ray_origin: Vec3,
                pub ray_direction: Vec3,
                pub select: bool,
                pub freeze: bool,
                pub translate_to: Option<Vec3>,
            }
            impl Input {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    aim_delta: impl Into<Vec2>,
                    movement: impl Into<Vec2>,
                    boost: impl Into<bool>,
                    ray_origin: impl Into<Vec3>,
                    ray_direction: impl Into<Vec3>,
                    select: impl Into<bool>,
                    freeze: impl Into<bool>,
                    translate_to: impl Into<Option<Vec3>>,
                ) -> Self {
                    Self {
                        aim_delta: aim_delta.into(),
                        movement: movement.into(),
                        boost: boost.into(),
                        ray_origin: ray_origin.into(),
                        ray_direction: ray_direction.into(),
                        select: select.into(),
                        freeze: freeze.into(),
                        translate_to: translate_to.into(),
                    }
                }
            }
            impl Message for Input {
                fn id() -> &'static str {
                    "xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::Input"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.aim_delta.serialize_message_part(&mut output)?;
                    self.movement.serialize_message_part(&mut output)?;
                    self.boost.serialize_message_part(&mut output)?;
                    self.ray_origin.serialize_message_part(&mut output)?;
                    self.ray_direction.serialize_message_part(&mut output)?;
                    self.select.serialize_message_part(&mut output)?;
                    self.freeze.serialize_message_part(&mut output)?;
                    self.translate_to.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        aim_delta: Vec2::deserialize_message_part(&mut input)?,
                        movement: Vec2::deserialize_message_part(&mut input)?,
                        boost: bool::deserialize_message_part(&mut input)?,
                        ray_origin: Vec3::deserialize_message_part(&mut input)?,
                        ray_direction: Vec3::deserialize_message_part(&mut input)?,
                        select: bool::deserialize_message_part(&mut input)?,
                        freeze: bool::deserialize_message_part(&mut input)?,
                        translate_to: Option::<Vec3>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Input {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
}
pub use raw::n7xfnlfzdmnvj7bqasfdhqftbtdi27ah as editor_schema;
pub use raw::xar372tfo2oswb4pkvx7h7o3rxi6tap6 as hide_cursor;
pub use raw::xr6whcy65gn3vlzrp2ssyn7udcbxb6mu as this;
