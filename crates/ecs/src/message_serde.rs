use std::io::Read;

use ambient_package_rt::message_serde::{MessageSerde, MessageSerdeError};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::EntityId;

// Keep in sync with [the Rust guest](guest/rust/api_core/src/message/serde.rs).

impl MessageSerde for EntityId {
    fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
        output.write_u128::<BigEndian>(self.0)?;
        Ok(())
    }

    fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
        Ok(Self(input.read_u128::<BigEndian>()?))
    }
}
