use std::fmt::Debug;

use crate::{
    global::{CallbackReturn, EntityId},
    internal::{conversion::FromBindgen, executor::EXECUTOR, wit},
};

mod serde;
pub use self::serde::*;

#[cfg(any(feature = "client", feature = "server"))]
use crate::internal::conversion::IntoBindgen;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Where a message came from.
pub enum Source {
    /// This message came from the runtime.
    Runtime,
    /// This message came from the corresponding serverside package.
    #[cfg(feature = "client")]
    Server,
    /// This message came from the corresponding clientside package and was sent from `user_id`.
    #[cfg(feature = "server")]
    Client {
        /// The user that sent this message.
        user_id: String,
    },
    /// This message came from another package on this side.
    Local(EntityId),
}
impl Source {
    /// Is this message from the runtime?
    pub fn runtime(&self) -> bool {
        matches!(self, Source::Runtime)
    }

    #[cfg(feature = "client")]
    /// Is this message from the corresponding serverside package?
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
        let Some(user_id) = self.client_user_id() else {
            return None;
        };
        let Some(player_id) = crate::player::get_by_user_id(&user_id) else {
            return None;
        };
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
    /// A message to all other packages running on this side.
    LocalBroadcast {
        /// Whether or not the message should be sent to the package that originally sent the message.
        include_self: bool,
    },
    /// A message to a specific package or module running on this side.
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
    /// Note that this message will only be received by the corresponding package
    /// on the server, and not by any other packages. You will need to explicitly
    /// relay the message to other packages on the server.
    #[cfg(feature = "client")]
    ServerUnreliable,
    /// A reliable transmission to the server (guaranteed to be received).
    ///
    /// Reliable messages are implemented using QUIC streams. This makes them ideal
    /// for messages that are sent infrequently, but must be received by the server.
    ///
    /// Note that this message will only be received by the corresponding package
    /// on the server, and not by any other packages. You will need to explicitly
    /// relay the message to other packages on the server.
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
    /// Note that this message will only be received by the corresponding package
    /// on the client, and not by any other packages. You will need to explicitly
    /// relay the message to other packages on the client.
    #[cfg(feature = "server")]
    ClientBroadcastUnreliable,
    /// A reliable transmission to all clients (guaranteed to be received).
    ///
    /// Reliable messages are implemented using QUIC streams. This makes them ideal
    /// for messages that are sent infrequently, but must be received by the client.
    ///
    /// Note that this message will only be received by the corresponding package
    /// on the client, and not by any other packages. You will need to explicitly
    /// relay the message to other packages on the client.
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
    /// Note that this message will only be received by the corresponding package
    /// on the client, and not by any other packages. You will need to explicitly
    /// relay the message to other packages on the client.
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
    /// Note that this message will only be received by the corresponding package
    /// on the client, and not by any other packages. You will need to explicitly
    /// relay the message to other packages on the client.
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

/// Send a message from this package to a specific `target`.
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
    pub fn stop(&self) {
        EXECUTOR.unregister_callback(&self.0, self.1);
    }
}

/// Message context.
pub struct MessageContext {
    /// Where the message came from.
    pub source: Source,
    /// The listener that can be used to stop listening.
    pub listener: Listener,
}
impl MessageContext {
    /// Is this message from the runtime?
    pub fn runtime(&self) -> bool {
        self.source.runtime()
    }

    #[cfg(feature = "client")]
    /// Is this message from the corresponding serverside package?
    pub fn server(&self) -> bool {
        self.source.server()
    }

    #[cfg(feature = "server")]
    /// The user that sent this message, if any.
    pub fn client_user_id(&self) -> Option<String> {
        self.source.client_user_id()
    }

    #[cfg(feature = "server")]
    /// The entity ID of the player that sent this message, if any.
    pub fn client_entity_id(&self) -> Option<EntityId> {
        self.source.client_entity_id()
    }

    /// The module on this side that sent this message, if any.
    pub fn local(&self) -> Option<EntityId> {
        self.source.local()
    }

    /// Stops listening.
    pub fn stop(&self) {
        self.listener.stop()
    }
}
impl Debug for MessageContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}

/// Subscribes to a message.
///
/// To unsubscribe from a message, call [Listener::stop] on the returned [Listener],
/// or on the [MessageContext] that is passed to the callback.
#[allow(clippy::collapsible_else_if)]
pub fn subscribe<R: CallbackReturn, T: Message>(
    mut callback: impl FnMut(MessageContext, T) -> R + 'static,
) -> Listener {
    let id = T::id();
    wit::message::subscribe(id);
    Listener(
        id.to_string(),
        EXECUTOR.register_callback(
            id.to_string(),
            Box::new(move |source, listener_id, data| {
                callback(
                    MessageContext {
                        source: source.clone().from_bindgen(),
                        listener: Listener(id.to_string(), listener_id),
                    },
                    T::deserialize_message(data)?,
                )
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

    /// Sends a message to every package on this side.
    ///
    /// `include_self` controls whether or not the message is sent to the package that originally sent the message.
    fn send_local_broadcast(&self, include_self: bool) {
        self.send(Target::LocalBroadcast { include_self })
    }

    /// Sends a message to a specific package or module on this side.
    fn send_local(&self, target_id: EntityId) {
        self.send(Target::Local(target_id))
    }

    #[cfg(feature = "client")]
    /// Sends an unreliable message to the server.
    ///
    /// Note that this message will only be received by the corresponding package on the server,
    /// and not by any other packages. You will need to explicitly relay the message to other
    /// packages on the server.
    ///
    /// See [Target::ServerUnreliable] for details.
    fn send_server_unreliable(&self) {
        self.send(Target::ServerUnreliable)
    }

    #[cfg(feature = "client")]
    /// Sends a reliable message to the server.
    ///
    /// Note that this message will only be received by the corresponding package on the server,
    /// and not by any other packages. You will need to explicitly relay the message to other
    /// packages on the server.
    ///
    /// See [Target::ServerReliable] for details.
    fn send_server_reliable(&self) {
        self.send(Target::ServerReliable)
    }

    #[cfg(feature = "server")]
    /// Sends an unreliable message to all clients.
    ///
    /// Note that this message will only be received by the corresponding package on the client,
    /// and not by any other packages. You will need to explicitly relay the message to other
    /// packages on the client.
    ///
    /// See [Target::ClientBroadcastUnreliable] for details.
    fn send_client_broadcast_unreliable(&self) {
        self.send(Target::ClientBroadcastUnreliable)
    }

    #[cfg(feature = "server")]
    /// Sends a reliable message to all clients.
    ///
    /// Note that this message will only be received by the corresponding package on the client,
    /// and not by any other packages. You will need to explicitly relay the message to other
    /// packages on the client.
    ///
    /// See [Target::ClientBroadcastReliable] for details.
    fn send_client_broadcast_reliable(&self) {
        self.send(Target::ClientBroadcastReliable)
    }

    #[cfg(feature = "server")]
    /// Sends an unreliable message to a specific client.
    ///
    /// Note that this message will only be received by the corresponding package on the client,
    /// and not by any other packages. You will need to explicitly relay the message to other
    /// packages on the client.
    ///
    /// See [Target::ClientTargetedUnreliable] for details.
    fn send_client_targeted_unreliable(&self, user_id: String) {
        self.send(Target::ClientTargetedUnreliable(user_id))
    }

    #[cfg(feature = "server")]
    /// Sends a reliable message to a specific client.
    ///
    /// Note that this message will only be received by the corresponding package on the client,
    /// and not by any other packages. You will need to explicitly relay the message to other
    /// packages on the client.
    ///
    /// See [Target::ClientTargetedReliable] for details.
    fn send_client_targeted_reliable(&self, user_id: String) {
        self.send(Target::ClientTargetedReliable(user_id))
    }

    /// Subscribes to this [Message]. Wrapper around [self::subscribe].
    fn subscribe<R: CallbackReturn>(
        callback: impl FnMut(MessageContext, Self) -> R + 'static,
    ) -> Listener {
        self::subscribe(callback)
    }
}

/// Implemented by all messages sent from the runtime.
pub trait RuntimeMessage: Message {
    /// Subscribes to this [Message]. Wrapper around [self::subscribe].
    fn subscribe<R: CallbackReturn>(mut callback: impl FnMut(Self) -> R + 'static) -> Listener {
        self::subscribe(move |_ctx, msg| callback(msg))
    }
}
