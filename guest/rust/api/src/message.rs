#[cfg(feature = "client")]
/// Client-specific message functionality.
pub mod client {
    use crate::{
        components::core::wasm::message::{data, source_module, source_network},
        event,
        global::{on, EntityId},
        internal::{conversion::IntoBindgen, wit},
        prelude::EventResult,
    };

    use super::Message;

    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    /// Where a message came from.
    pub enum Source {
        /// This message came from the corresponding serverside module.
        Network,
        /// This message came from another serverside module.
        Module(EntityId),
    }

    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    /// The target for a message.
    pub enum Target {
        /// An unreliable transmission to the server.
        ///
        /// Not guaranteed to be received, and must be below one kilobyte.
        NetworkUnreliable,
        /// A reliable transmission to the server (guaranteed to be received).
        NetworkReliable,
        /// A message to all other modules running on this client.
        ModuleBroadcast,
        /// A message to a specific module running on this client.
        Module(EntityId),
    }

    impl IntoBindgen for Target {
        type Item = wit::client_message::Target;

        fn into_bindgen(self) -> Self::Item {
            match self {
                Target::NetworkUnreliable => Self::Item::NetworkUnreliable,
                Target::NetworkReliable => Self::Item::NetworkReliable,
                Target::ModuleBroadcast => Self::Item::ModuleBroadcast,
                Target::Module(id) => Self::Item::Module(id.into_bindgen()),
            }
        }
    }

    /// Send a message from this client module to a specific `target`.
    pub fn send<T: Message>(target: Target, data: &T) {
        wit::client_message::send(
            target.into_bindgen(),
            T::id(),
            &data.serialize_message().unwrap(),
        )
    }

    /// Subscribes to a message.
    pub fn subscribe<T: Message>(callback: impl FnMut(Source, T) -> EventResult + 'static) {
        let mut callback = Box::new(callback);
        on(
            &format!("{}/{}", event::MODULE_MESSAGE, T::id()),
            move |e| {
                let source = if e.get(source_network()).is_some() {
                    Source::Network
                } else if let Some(module) = e.get(source_module()) {
                    Source::Module(module)
                } else {
                    panic!("No source available for incoming message");
                };

                let data = e.get(data()).expect("No data for incoming message");

                callback(source, T::deserialize_message(&data)?)
            },
        );
    }

    /// Adds helpers for sending/subscribing to [Message]s.
    pub trait MessageExt: Message {
        /// Sends this [Message] to `target`. Wrapper around [self::send].
        fn send(&self, target: Target) {
            self::send(target, self)
        }

        /// Subscribes to this [Message]. Wrapper around [self::subscribe].
        fn subscribe(callback: impl FnMut(Source, Self) -> EventResult + 'static) {
            self::subscribe(callback)
        }
    }
    impl<T: Message> MessageExt for T {}
}

#[cfg(feature = "server")]
/// Server-specific message functionality.
pub mod server {
    use crate::{
        components::core::wasm::message::{data, source_module, source_network_user_id},
        event,
        global::{on, EntityId, EventResult},
        internal::{conversion::IntoBindgen, wit},
    };

    use super::Message;

    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    /// Where a message came from.
    pub enum Source {
        /// This message came from the corresponding clientside module and was sent from `user_id`.
        Network {
            /// The user that sent this message.
            user_id: String,
        },
        /// This message came from another serverside module.
        Module(EntityId),
    }

    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    /// The target for a message.
    pub enum Target {
        /// An unreliable transmission to all clients.
        ///
        /// Not guaranteed to be received, and must be below one kilobyte.
        NetworkBroadcastUnreliable,
        /// A reliable transmission to all clients (guaranteed to be received).
        NetworkBroadcastReliable,
        /// An unreliable transmission to a specific client.
        ///
        /// Not guaranteed to be received, and must be below one kilobyte.
        NetworkTargetedUnreliable(
            /// The user to send to.
            String,
        ),
        /// A reliable transmission to a specific client (guaranteed to be received).
        NetworkTargetedReliable(
            /// The user to send to.
            String,
        ),
        /// A message to all other modules running on this server.
        ModuleBroadcast,
        /// A message to a specific module running on this server.
        Module(EntityId),
    }

    impl<'a> IntoBindgen for &'a Target {
        type Item = wit::server_message::Target<'a>;

        fn into_bindgen(self) -> Self::Item {
            match self {
                Target::NetworkBroadcastUnreliable => Self::Item::NetworkBroadcastUnreliable,
                Target::NetworkBroadcastReliable => Self::Item::NetworkBroadcastReliable,
                Target::NetworkTargetedUnreliable(user_id) => {
                    Self::Item::NetworkTargetedUnreliable(user_id.as_str())
                }
                Target::NetworkTargetedReliable(user_id) => {
                    Self::Item::NetworkTargetedReliable(user_id.as_str())
                }
                Target::ModuleBroadcast => Self::Item::ModuleBroadcast,
                Target::Module(id) => Self::Item::Module(id.into_bindgen()),
            }
        }
    }

    /// Send a message from this server module to a specific `target`.
    pub fn send<T: Message>(target: Target, data: &T) {
        wit::server_message::send(
            target.into_bindgen(),
            T::id(),
            &data.serialize_message().unwrap(),
        )
    }

    /// Subscribes to a message.
    pub fn subscribe<T: Message>(callback: impl FnMut(Source, T) -> EventResult + 'static) {
        let mut callback = Box::new(callback);
        on(
            &format!("{}/{}", event::MODULE_MESSAGE, T::id()),
            move |e| {
                let source = if let Some(user_id) = e.get(source_network_user_id()) {
                    Source::Network { user_id }
                } else if let Some(module) = e.get(source_module()) {
                    Source::Module(module)
                } else {
                    panic!("No source available for incoming message");
                };

                let data = e.get(data()).expect("No data for incoming message");

                callback(source, T::deserialize_message(&data)?)
            },
        );
    }

    /// Adds helpers for sending/subscribing to [Message]s.
    pub trait MessageExt: Message {
        /// Sends this [Message] to `target`. Wrapper around [self::send].
        fn send(&self, target: Target) {
            self::send(target, self)
        }

        /// Subscribes to this [Message]. Wrapper around [self::subscribe].
        fn subscribe(callback: impl FnMut(Source, Self) -> EventResult + 'static) {
            self::subscribe(callback)
        }
    }
    impl<T: Message> MessageExt for T {}
}

mod serde {
    pub use ambient_project::message_serde::*;

    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

    use crate::global::EntityId;

    impl MessageSerde for EntityId {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            output.write_u64::<BigEndian>(self.id0)?;
            output.write_u64::<BigEndian>(self.id1)?;
            Ok(())
        }

        fn deserialize_message_part(
            input: &mut dyn std::io::Read,
        ) -> Result<Self, MessageSerdeError> {
            let (id0, id1) = (
                input.read_u64::<BigEndian>()?,
                input.read_u64::<BigEndian>()?,
            );
            Ok(Self { id0, id1 })
        }
    }
}
pub use serde::*;
