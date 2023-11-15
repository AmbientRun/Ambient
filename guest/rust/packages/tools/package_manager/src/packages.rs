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
    pub mod hr4pxz7kfhzgimicoyh65ydel3aehuhk {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("hr4pxz7kfhzgimicoyh65ydel3aehuhk")
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
            static MOD_MANAGER_FOR: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("hr4pxz7kfhzgimicoyh65ydel3aehuhk::mod_manager_for")
            });
            #[doc = "**Mod Manager For**: Package config component. Attach this component to this package's entity to make it a mod manager for the given package.\n\n*Attributes*: Networked, Debuggable"]
            pub fn mod_manager_for() -> Component<EntityId> {
                *MOD_MANAGER_FOR
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
            #[doc = "**PackageLoad**"]
            pub struct PackageLoad {
                pub url: String,
                pub enabled: bool,
            }
            impl PackageLoad {
                #[allow(clippy::too_many_arguments)]
                pub fn new(url: impl Into<String>, enabled: impl Into<bool>) -> Self {
                    Self {
                        url: url.into(),
                        enabled: enabled.into(),
                    }
                }
            }
            impl Message for PackageLoad {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageLoad"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.url.serialize_message_part(&mut output)?;
                    self.enabled.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        url: String::deserialize_message_part(&mut input)?,
                        enabled: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageLoad {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageLoadSuccess**"]
            pub struct PackageLoadSuccess {
                pub id: EntityId,
                pub name: String,
            }
            impl PackageLoadSuccess {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>, name: impl Into<String>) -> Self {
                    Self {
                        id: id.into(),
                        name: name.into(),
                    }
                }
            }
            impl Message for PackageLoadSuccess {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageLoadSuccess"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    self.name.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                        name: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageLoadSuccess {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageLoadFailure**"]
            pub struct PackageLoadFailure {
                pub reason: String,
            }
            impl PackageLoadFailure {
                #[allow(clippy::too_many_arguments)]
                pub fn new(reason: impl Into<String>) -> Self {
                    Self {
                        reason: reason.into(),
                    }
                }
            }
            impl Message for PackageLoadFailure {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageLoadFailure"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.reason.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        reason: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageLoadFailure {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageSetEnabled**"]
            pub struct PackageSetEnabled {
                pub id: EntityId,
                pub enabled: bool,
            }
            impl PackageSetEnabled {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>, enabled: impl Into<bool>) -> Self {
                    Self {
                        id: id.into(),
                        enabled: enabled.into(),
                    }
                }
            }
            impl Message for PackageSetEnabled {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageSetEnabled"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    self.enabled.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                        enabled: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageSetEnabled {}
            #[derive(Clone, Debug)]
            #[doc = "**WasmSetEnabled**"]
            pub struct WasmSetEnabled {
                pub id: EntityId,
                pub enabled: bool,
            }
            impl WasmSetEnabled {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>, enabled: impl Into<bool>) -> Self {
                    Self {
                        id: id.into(),
                        enabled: enabled.into(),
                    }
                }
            }
            impl Message for WasmSetEnabled {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::WasmSetEnabled"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    self.enabled.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                        enabled: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for WasmSetEnabled {}
            #[derive(Clone, Debug)]
            #[doc = "**WasmReload**"]
            pub struct WasmReload {
                pub id: EntityId,
            }
            impl WasmReload {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>) -> Self {
                    Self { id: id.into() }
                }
            }
            impl Message for WasmReload {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::WasmReload"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for WasmReload {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageShow**: Shows the package view for a package"]
            pub struct PackageShow {
                pub id: EntityId,
            }
            impl PackageShow {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>) -> Self {
                    Self { id: id.into() }
                }
            }
            impl Message for PackageShow {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageShow"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageShow {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageLoadShow**: Shows the package load view"]
            pub struct PackageLoadShow;
            impl PackageLoadShow {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for PackageLoadShow {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageLoadShow"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for PackageLoadShow {}
            impl Default for PackageLoadShow {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**PackageRemoteRequest**: Requests all relevant remote packages"]
            pub struct PackageRemoteRequest;
            impl PackageRemoteRequest {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for PackageRemoteRequest {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageRemoteRequest"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for PackageRemoteRequest {}
            impl Default for PackageRemoteRequest {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**PackageRemoteResponse**: Response to a remote package request"]
            pub struct PackageRemoteResponse {
                pub packages: Vec<String>,
                pub error: Option<String>,
            }
            impl PackageRemoteResponse {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    packages: impl Into<Vec<String>>,
                    error: impl Into<Option<String>>,
                ) -> Self {
                    Self {
                        packages: packages.into(),
                        error: error.into(),
                    }
                }
            }
            impl Message for PackageRemoteResponse {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageRemoteResponse"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.packages.serialize_message_part(&mut output)?;
                    self.error.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        packages: Vec::<String>::deserialize_message_part(&mut input)?,
                        error: Option::<String>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageRemoteResponse {}
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
}
pub use raw::hr4pxz7kfhzgimicoyh65ydel3aehuhk as this;
pub use raw::n7xfnlfzdmnvj7bqasfdhqftbtdi27ah as editor_schema;
