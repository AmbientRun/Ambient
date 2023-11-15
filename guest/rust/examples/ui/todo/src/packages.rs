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
    pub mod jruv4zn5tfe2s3kpyhhq7z3uezbi27cv {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("jruv4zn5tfe2s3kpyhhq7z3uezbi27cv")
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
            static TODO_ITEM: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("jruv4zn5tfe2s3kpyhhq7z3uezbi27cv::todo_item")
            });
            #[doc = "**Todo item**: Item in the todo list.\n\n*Attributes*: Networked, Debuggable"]
            pub fn todo_item() -> Component<String> {
                *TODO_ITEM
            }
            static TODO_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component("jruv4zn5tfe2s3kpyhhq7z3uezbi27cv::todo_time")
            });
            #[doc = "**Todo time**: The time the todo was created.\n\n*Attributes*: Networked, Debuggable"]
            pub fn todo_time() -> Component<Duration> {
                *TODO_TIME
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
            #[doc = "**NewItem**: Add a new todo item"]
            pub struct NewItem {
                pub description: String,
            }
            impl NewItem {
                #[allow(clippy::too_many_arguments)]
                pub fn new(description: impl Into<String>) -> Self {
                    Self {
                        description: description.into(),
                    }
                }
            }
            impl Message for NewItem {
                fn id() -> &'static str {
                    "jruv4zn5tfe2s3kpyhhq7z3uezbi27cv::NewItem"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.description.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        description: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for NewItem {}
            #[derive(Clone, Debug)]
            #[doc = "**DeleteItem**: Remove todo item"]
            pub struct DeleteItem {
                pub id: EntityId,
            }
            impl DeleteItem {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>) -> Self {
                    Self { id: id.into() }
                }
            }
            impl Message for DeleteItem {
                fn id() -> &'static str {
                    "jruv4zn5tfe2s3kpyhhq7z3uezbi27cv::DeleteItem"
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
            impl ModuleMessage for DeleteItem {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
}
pub use raw::jruv4zn5tfe2s3kpyhhq7z3uezbi27cv as this;
