#[cfg(feature = "client")]
mod client {
    use crate::{
        global::EntityId,
        internal::{conversion::IntoBindgen, wit},
    };

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
    pub fn send(target: Target, name: impl AsRef<str>, data: impl Into<Vec<u8>>) {
        wit::client_message::send(target.into_bindgen(), name.as_ref(), &data.into())
    }
}

#[cfg(feature = "client")]
pub use client::*;
