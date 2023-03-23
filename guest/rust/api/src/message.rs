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

mod message_serde {
    use std::io::Read;

    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
    use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
    use thiserror::Error;

    use crate::global::EntityId;

    #[derive(Error, Debug)]
    /// Error that can occur during message ser/de.
    pub enum MessageSerdeError {
        /// Arbitrary I/O error during ser/de.
        #[error("arbitrary I/O error")]
        IO(#[from] std::io::Error),
        /// An invalid value was encountered during ser/de.
        #[error("invalid value")]
        InvalidValue,
        /// The length of an array exceeded 2^32-1 bytes.
        #[error("array too long")]
        ArrayTooLong(#[from] std::num::TryFromIntError),
    }

    /// Implemented for all types that can be serialized in a message.
    pub trait MessageSerde: Default + Clone
    where
        Self: Sized,
    {
        /// Serialize this to a Vec<u8>.
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError>;
        /// Deserialize this if possible.
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError>;
    }
    impl MessageSerde for () {
        fn serialize_message_part(&self, _output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            Ok(())
        }
        fn deserialize_message_part(_input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            Ok(())
        }
    }
    impl MessageSerde for bool {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            Ok(output.write_u8(if *self { 1 } else { 0 })?)
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            match input.read_u8()? {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(MessageSerdeError::InvalidValue),
            }
        }
    }
    impl MessageSerde for EntityId {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            output.write_u64::<BigEndian>(self.id0)?;
            output.write_u64::<BigEndian>(self.id1)?;
            Ok(())
        }

        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            Ok(Self {
                id0: input.read_u64::<BigEndian>()?,
                id1: input.read_u64::<BigEndian>()?,
            })
        }
    }
    impl MessageSerde for f32 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            Ok(output.write_f32::<BigEndian>(*self)?)
        }

        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            Ok(input.read_f32::<BigEndian>()?)
        }
    }
    impl MessageSerde for f64 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            Ok(output.write_f64::<BigEndian>(*self)?)
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            Ok(input.read_f64::<BigEndian>()?)
        }
    }
    impl MessageSerde for Mat4 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            for value in self.to_cols_array() {
                output.write_f32::<BigEndian>(value)?;
            }
            Ok(())
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            let mut values = [0f32; 16];
            for value in &mut values {
                *value = input.read_f32::<BigEndian>()?;
            }
            Ok(Self::from_cols_array(&values))
        }
    }
    impl MessageSerde for i32 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            Ok(output.write_i32::<BigEndian>(*self)?)
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            Ok(input.read_i32::<BigEndian>()?)
        }
    }
    impl MessageSerde for Quat {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            for value in self.to_array() {
                output.write_f32::<BigEndian>(value)?;
            }
            Ok(())
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            let mut values = [0f32; 4];
            for value in &mut values {
                *value = input.read_f32::<BigEndian>()?;
            }
            Ok(Self::from_array(values))
        }
    }
    impl MessageSerde for u8 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            Ok(output.write_u8(*self)?)
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            Ok(input.read_u8()?)
        }
    }
    impl MessageSerde for u32 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            Ok(output.write_u32::<BigEndian>(*self)?)
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            Ok(input.read_u32::<BigEndian>()?)
        }
    }
    impl MessageSerde for u64 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            Ok(output.write_u64::<BigEndian>(*self)?)
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            Ok(input.read_u64::<BigEndian>()?)
        }
    }
    impl MessageSerde for Vec2 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            for value in self.to_array() {
                output.write_f32::<BigEndian>(value)?;
            }
            Ok(())
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            let mut values = [0f32; 2];
            for value in &mut values {
                *value = input.read_f32::<BigEndian>()?;
            }
            Ok(Self::from_array(values))
        }
    }
    impl MessageSerde for Vec3 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            for value in self.to_array() {
                output.write_f32::<BigEndian>(value)?;
            }
            Ok(())
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            let mut values = [0f32; 3];
            for value in &mut values {
                *value = input.read_f32::<BigEndian>()?;
            }
            Ok(Self::from_array(values))
        }
    }
    impl MessageSerde for Vec4 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            for value in self.to_array() {
                output.write_f32::<BigEndian>(value)?;
            }
            Ok(())
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            let mut values = [0f32; 4];
            for value in &mut values {
                *value = input.read_f32::<BigEndian>()?;
            }
            Ok(Self::from_array(values))
        }
    }
    impl MessageSerde for UVec2 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            for value in self.to_array() {
                output.write_u32::<BigEndian>(value)?;
            }
            Ok(())
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            let mut values = [0u32; 2];
            for value in &mut values {
                *value = input.read_u32::<BigEndian>()?;
            }
            Ok(Self::from_array(values))
        }
    }
    impl MessageSerde for UVec3 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            for value in self.to_array() {
                output.write_u32::<BigEndian>(value)?;
            }
            Ok(())
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            let mut values = [0u32; 3];
            for value in &mut values {
                *value = input.read_u32::<BigEndian>()?;
            }
            Ok(Self::from_array(values))
        }
    }
    impl MessageSerde for UVec4 {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            for value in self.to_array() {
                output.write_u32::<BigEndian>(value)?;
            }
            Ok(())
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            let mut values = [0u32; 4];
            for value in &mut values {
                *value = input.read_u32::<BigEndian>()?;
            }
            Ok(Self::from_array(values))
        }
    }
    impl MessageSerde for String {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            serialize_array(output, self.as_bytes())
        }
        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            String::from_utf8(deserialize_array(input)?)
                .map_err(|_| MessageSerdeError::InvalidValue)
        }
    }
    impl<T: MessageSerde> MessageSerde for Vec<T> {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            serialize_array(output, self.as_slice())
        }

        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            deserialize_array(input)
        }
    }
    impl<T: MessageSerde> MessageSerde for Option<T> {
        fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
            if let Some(value) = self {
                true.serialize_message_part(output)?;
                value.serialize_message_part(output)?;
            } else {
                false.serialize_message_part(output)?;
            }
            Ok(())
        }

        fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
            let present = bool::deserialize_message_part(input)?;
            Ok(if present {
                Some(T::deserialize_message_part(input)?)
            } else {
                None
            })
        }
    }

    fn serialize_array<T: MessageSerde>(
        output: &mut Vec<u8>,
        data: &[T],
    ) -> Result<(), MessageSerdeError> {
        output.write_u32::<BigEndian>(data.len().try_into()?)?;
        for value in data {
            value.serialize_message_part(output)?;
        }
        Ok(())
    }

    fn deserialize_array<T: MessageSerde>(
        input: &mut dyn Read,
    ) -> Result<Vec<T>, MessageSerdeError> {
        let length = usize::try_from(input.read_u32::<BigEndian>()?).unwrap();
        let mut data = vec![Default::default(); length];
        for value in &mut data {
            *value = T::deserialize_message_part(input)?;
        }
        Ok(data)
    }

    /// Implemented on all types that can be de/serialized from/to a Vec<u8>.
    pub trait Message: Sized {
        /// The identifier of this message.
        fn id() -> &'static str;

        /// Serialize this to a Vec<u8>.
        fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError>;
        /// Deserialize this from a u8 slice.
        fn deserialize_message(input: &[u8]) -> Result<Self, MessageSerdeError>;
    }
}
pub use message_serde::*;
