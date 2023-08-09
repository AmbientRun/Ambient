use crate::{
    global::{CallbackReturn, EntityId},
    internal::{conversion::FromBindgen, executor::EXECUTOR, wit},
};

#[cfg(any(feature = "client", feature = "server"))]
use crate::internal::conversion::IntoBindgen;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Where a message came from.
pub enum Source {
    /// This message came from the runtime.
    Runtime,
    /// This message came from the corresponding serverside module.
    #[cfg(feature = "client")]
    Server,
    /// This message came from the corresponding clientside module and was sent from `user_id`.
    #[cfg(feature = "server")]
    Client {
        /// The user that sent this message.
        user_id: String,
    },
    /// This message came from another module on this side.
    Local(EntityId),
}
impl Source {
    /// Is this message from the runtime?
    pub fn runtime(&self) -> bool {
        matches!(self, Source::Runtime)
    }

    #[cfg(feature = "client")]
    /// Is this message from the corresponding serverside module?
    pub fn server(&self) -> bool {
        matches!(self, Source::Server)
    }

    #[cfg(feature = "server")]
    /// The user that sent this message, if any.
    pub fn client_user_id(&self) -> Option<String> {
        if let Source::Client { user_id } = self {
            Some(user_id.clone())
        } else {
            None
        }
    }

    #[cfg(feature = "server")]
    /// The entity ID of the player that sent this message, if any.
    pub fn client_entity_id(&self) -> Option<EntityId> {
        let Some(user_id) = self.client_user_id() else { return None; };
        let Some(player_id) = crate::player::get_by_user_id(&user_id) else { return None; };
        Some(player_id)
    }

    /// The module on this side that sent this message, if any.
    pub fn local(&self) -> Option<EntityId> {
        match self {
            Source::Local(id) => Some(*id),
            _ => None,
        }
    }
}
impl FromBindgen for wit::guest::Source {
    type Item = Source;

    fn from_bindgen(self) -> Self::Item {
        match self {
            wit::guest::Source::Runtime => Source::Runtime,
            #[cfg(feature = "client")]
            wit::guest::Source::Server => Source::Server,
            #[cfg(feature = "server")]
            wit::guest::Source::Client(user_id) => Source::Client { user_id },
            wit::guest::Source::Local(entity_id) => Source::Local(entity_id.from_bindgen()),

            // cover the other features
            #[cfg(not(feature = "client"))]
            wit::guest::Source::Server => unreachable!(),
            #[cfg(not(feature = "server"))]
            wit::guest::Source::Client(_user_id) => unreachable!(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// The target for a originating message.
pub enum Target {
    /// A message to all other modules running on this side.
    LocalBroadcast {
        /// Whether or not the message should be sent to the module that originally sent the message.
        include_self: bool,
    },
    /// A message to a specific module running on this side.
    Local(EntityId),

    // Client
    /// An unreliable transmission to the server.
    ///
    /// Not guaranteed to be received, and must be below one kilobyte.
    ///
    /// Unreliable messages are implemented using QUIC datagrams. This makes them ideal
    /// for messages that are sent frequently, but are not critical to the functioning
    /// of the logic on the server.
    ///
    /// Note that this message will only be received by the corresponding module
    /// on the server, and not by any other modules. You will need to explicitly
    /// relay the message to other modules on the server.
    #[cfg(feature = "client")]
    ServerUnreliable,
    /// A reliable transmission to the server (guaranteed to be received).
    ///
    /// Reliable messages are implemented using QUIC streams. This makes them ideal
    /// for messages that are sent infrequently, but must be received by the server.
    ///
    /// Note that this message will only be received by the corresponding module
    /// on the server, and not by any other modules. You will need to explicitly
    /// relay the message to other modules on the server.
    #[cfg(feature = "client")]
    ServerReliable,

    // Server
    /// An unreliable transmission to all clients.
    ///
    /// Not guaranteed to be received, and must be below one kilobyte.
    ///
    /// Unreliable messages are implemented using QUIC datagrams. This makes them ideal
    /// for messages that are sent frequently, but are not critical to the functioning
    /// of the logic on the client.
    ///
    /// Note that this message will only be received by the corresponding module
    /// on the client, and not by any other modules. You will need to explicitly
    /// relay the message to other modules on the client.
    #[cfg(feature = "server")]
    ClientBroadcastUnreliable,
    /// A reliable transmission to all clients (guaranteed to be received).
    ///
    /// Reliable messages are implemented using QUIC streams. This makes them ideal
    /// for messages that are sent infrequently, but must be received by the client.
    ///
    /// Note that this message will only be received by the corresponding module
    /// on the client, and not by any other modules. You will need to explicitly
    /// relay the message to other modules on the client.
    #[cfg(feature = "server")]
    ClientBroadcastReliable,
    /// An unreliable transmission to a specific client.
    ///
    /// Not guaranteed to be received, and must be below one kilobyte.
    ///
    /// Unreliable messages are implemented using QUIC datagrams. This makes them ideal
    /// for messages that are sent frequently, but are not critical to the functioning
    /// of the logic on the client.
    ///
    /// Note that this message will only be received by the corresponding module
    /// on the client, and not by any other modules. You will need to explicitly
    /// relay the message to other modules on the client.
    #[cfg(feature = "server")]
    ClientTargetedUnreliable(
        /// The user to send to.
        String,
    ),
    /// A reliable transmission to a specific client (guaranteed to be received).
    ///
    /// Reliable messages are implemented using QUIC streams. This makes them ideal
    /// for messages that are sent infrequently, but must be received by the client.
    ///
    /// Note that this message will only be received by the corresponding module
    /// on the client, and not by any other modules. You will need to explicitly
    /// relay the message to other modules on the client.
    #[cfg(feature = "server")]
    ClientTargetedReliable(
        /// The user to send to.
        String,
    ),
}

#[cfg(feature = "client")]
impl IntoBindgen for Target {
    type Item = wit::client_message::Target;

    fn into_bindgen(self) -> Self::Item {
        match self {
            Target::ServerUnreliable => Self::Item::ServerUnreliable,
            Target::ServerReliable => Self::Item::ServerReliable,
            Target::LocalBroadcast { include_self } => Self::Item::LocalBroadcast(include_self),
            Target::Local(id) => Self::Item::Local(id.into_bindgen()),
            #[cfg(feature = "server")]
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "server")]
impl<'a> IntoBindgen for &'a Target {
    type Item = wit::server_message::Target;

    fn into_bindgen(self) -> Self::Item {
        match self {
            Target::ClientBroadcastUnreliable => Self::Item::ClientBroadcastUnreliable,
            Target::ClientBroadcastReliable => Self::Item::ClientBroadcastReliable,
            Target::ClientTargetedUnreliable(user_id) => {
                Self::Item::ClientTargetedUnreliable(user_id.clone())
            }
            Target::ClientTargetedReliable(user_id) => {
                Self::Item::ClientTargetedReliable(user_id.clone())
            }
            Target::LocalBroadcast { include_self } => Self::Item::LocalBroadcast(*include_self),
            Target::Local(id) => Self::Item::Local(id.into_bindgen()),
            #[cfg(feature = "client")]
            _ => unreachable!(),
        }
    }
}

/// Send a message from this module to a specific `target`.
pub fn send<T: Message>(target: Target, data: &T) {
    #[cfg(all(feature = "client", not(feature = "server")))]
    wit::client_message::send(
        target.into_bindgen(),
        T::id(),
        &data.serialize_message().unwrap(),
    );
    #[cfg(all(feature = "server", not(feature = "client")))]
    wit::server_message::send(
        &(&target).into_bindgen(),
        T::id(),
        &data.serialize_message().unwrap(),
    );
    #[cfg(any(
        all(not(feature = "server"), not(feature = "client")),
        all(feature = "server", feature = "client")
    ))]
    let _ = (target, data);
}

/// Handle to a message listener that can be used to stop listening.
pub struct Listener(String, u128);
impl Listener {
    /// Stops listening.
    pub fn stop(self) {
        EXECUTOR.unregister_callback(&self.0, self.1);
    }
}

/// Subscribes to a message.
#[allow(clippy::collapsible_else_if)]
pub fn subscribe<R: CallbackReturn, T: Message>(
    mut callback: impl FnMut(Source, T) -> R + 'static,
) -> Listener {
    let id = T::id();
    wit::message::subscribe(id);
    Listener(
        id.to_string(),
        EXECUTOR.register_callback(
            id.to_string(),
            Box::new(move |source, data| {
                callback(source.clone().from_bindgen(), T::deserialize_message(data)?)
                    .into_result()?;
                Ok(())
            }),
        ),
    )
}

/// Implemented by all messages that can be sent between modules.
pub trait ModuleMessage: Message {
    /// Sends this [Message] to `target`. Wrapper around [self::send].
    fn send(&self, target: Target) {
        self::send(target, self)
    }

    /// Sends a message to every module on this side.
    ///
    /// `include_self` controls whether or not the message is sent to the module that originally sent the message.
    fn send_local_broadcast(&self, include_self: bool) {
        self.send(Target::LocalBroadcast { include_self })
    }

    /// Sends a message to a specific module on this side.
    fn send_local(&self, module_id: EntityId) {
        self.send(Target::Local(module_id))
    }

    #[cfg(feature = "client")]
    /// Sends an unreliable message to the server.
    ///
    /// Note that this message will only be received by the corresponding module on the server,
    /// and not by any other modules. You will need to explicitly relay the message to other
    /// modules on the server.
    ///
    /// See [Target::ServerUnreliable] for details.
    fn send_server_unreliable(&self) {
        self.send(Target::ServerUnreliable)
    }

    #[cfg(feature = "client")]
    /// Sends a reliable message to the server.
    ///
    /// Note that this message will only be received by the corresponding module on the server,
    /// and not by any other modules. You will need to explicitly relay the message to other
    /// modules on the server.
    ///
    /// See [Target::ServerReliable] for details.
    fn send_server_reliable(&self) {
        self.send(Target::ServerReliable)
    }

    #[cfg(feature = "server")]
    /// Sends an unreliable message to all clients.
    ///
    /// Note that this message will only be received by the corresponding module on the client,
    /// and not by any other modules. You will need to explicitly relay the message to other
    /// modules on the client.
    ///
    /// See [Target::ClientBroadcastUnreliable] for details.
    fn send_client_broadcast_unreliable(&self) {
        self.send(Target::ClientBroadcastUnreliable)
    }

    #[cfg(feature = "server")]
    /// Sends a reliable message to all clients.
    ///
    /// Note that this message will only be received by the corresponding module on the client,
    /// and not by any other modules. You will need to explicitly relay the message to other
    /// modules on the client.
    ///
    /// See [Target::ClientBroadcastReliable] for details.
    fn send_client_broadcast_reliable(&self) {
        self.send(Target::ClientBroadcastReliable)
    }

    #[cfg(feature = "server")]
    /// Sends an unreliable message to a specific client.
    ///
    /// Note that this message will only be received by the corresponding module on the client,
    /// and not by any other modules. You will need to explicitly relay the message to other
    /// modules on the client.
    ///
    /// See [Target::ClientTargetedUnreliable] for details.
    fn send_client_targeted_unreliable(&self, user_id: String) {
        self.send(Target::ClientTargetedUnreliable(user_id))
    }

    #[cfg(feature = "server")]
    /// Sends a reliable message to a specific client.
    ///
    /// Note that this message will only be received by the corresponding module on the client,
    /// and not by any other modules. You will need to explicitly relay the message to other
    /// modules on the client.
    ///
    /// See [Target::ClientTargetedReliable] for details.
    fn send_client_targeted_reliable(&self, user_id: String) {
        self.send(Target::ClientTargetedReliable(user_id))
    }

    /// Subscribes to this [Message]. Wrapper around [self::subscribe].
    fn subscribe<R: CallbackReturn>(callback: impl FnMut(Source, Self) -> R + 'static) -> Listener {
        self::subscribe(callback)
    }
}

/// Implemented by all messages sent from the runtime.
pub trait RuntimeMessage: Message {
    /// Subscribes to this [Message]. Wrapper around [self::subscribe].
    fn subscribe<R: CallbackReturn>(mut callback: impl FnMut(Self) -> R + 'static) -> Listener {
        self::subscribe(move |_source, msg| callback(msg))
    }
}

mod serde {
    pub use ambient_project_rt::message_serde::*;

    use ambient_shared_types::procedural_storage_handle_definitions;
    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
    use paste::paste;

    use crate::global::{
        EntityId, ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
        ProceduralTextureHandle,
    };

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

    macro_rules! make_procedural_storage_handle_serializers {
        ($($name:ident),*) => { paste!{$(
            impl MessageSerde for [<Procedural $name:camel Handle>] {
                fn serialize_message_part(
                    &self,
                    output: &mut Vec<u8>,
                ) -> Result<(), MessageSerdeError> {
                    let ulid = self.0;
                    output.write_u64::<BigEndian>(ulid.0)?;
                    output.write_u64::<BigEndian>(ulid.1)?;
                    Ok(())
                }

                fn deserialize_message_part(
                    input: &mut dyn std::io::Read,
                ) -> Result<Self, MessageSerdeError> {
                    Ok(Self((
                        input.read_u64::<BigEndian>()?,
                        input.read_u64::<BigEndian>()?,
                    )))
                }
            }
        )*}};
    }

    procedural_storage_handle_definitions!(make_procedural_storage_handle_serializers);
}
pub use serde::*;
