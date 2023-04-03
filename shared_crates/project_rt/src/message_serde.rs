use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use thiserror::Error;

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
    /// Serialize this to a `Vec<u8>`.
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
#[cfg(feature = "native")]
impl MessageSerde for ambient_ecs::EntityId {
    fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
        let (id0, id1) = self.to_u64s();
        output.write_u64::<BigEndian>(id0)?;
        output.write_u64::<BigEndian>(id1)?;
        Ok(())
    }

    fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
        let (id0, id1) = (input.read_u64::<BigEndian>()?, input.read_u64::<BigEndian>()?);
        Ok(Self::from_u64s(id0, id1))
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
        String::from_utf8(deserialize_array(input)?).map_err(|_| MessageSerdeError::InvalidValue)
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
        Ok(if present { Some(T::deserialize_message_part(input)?) } else { None })
    }
}

fn serialize_array<T: MessageSerde>(output: &mut Vec<u8>, data: &[T]) -> Result<(), MessageSerdeError> {
    output.write_u32::<BigEndian>(data.len().try_into()?)?;
    for value in data {
        value.serialize_message_part(output)?;
    }
    Ok(())
}

fn deserialize_array<T: MessageSerde>(input: &mut dyn Read) -> Result<Vec<T>, MessageSerdeError> {
    let length = usize::try_from(input.read_u32::<BigEndian>()?).unwrap();
    let mut data = vec![Default::default(); length];
    for value in &mut data {
        *value = T::deserialize_message_part(input)?;
    }
    Ok(data)
}

/// Implemented on all types that can be de/serialized from/to a `Vec<u8>`.
pub trait Message: Sized {
    /// The identifier of this message.
    #[doc(hidden)]
    fn id() -> &'static str;

    /// Serialize this to a `Vec<u8>`.
    #[doc(hidden)]
    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError>;
    /// Deserialize this from a `u8` slice.
    #[doc(hidden)]
    fn deserialize_message(input: &[u8]) -> Result<Self, MessageSerdeError>;
}
