use std::io::Read;

use ambient_project_rt::message_serde::{MessageSerde, MessageSerdeError};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

impl MessageSerde for crate::EntityId {
    fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
        let (id0, id1) = self.to_u64s();
        output.write_u64::<BigEndian>(id0)?;
        output.write_u64::<BigEndian>(id1)?;
        Ok(())
    }

    fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
        let (id0, id1) = (
            input.read_u64::<BigEndian>()?,
            input.read_u64::<BigEndian>()?,
        );
        Ok(Self::from_u64s(id0, id1))
    }
}

mod inner {
    ambient_project_macro::host_project!();
}
// Legacy re-exports to maintain compatibility with old code
pub mod components {
    pub use super::inner::components::{ambient::*, init};
}
pub mod messages {
    pub use super::inner::messages::ambient::core::*;
}
pub mod concepts {
    // pub use super::inner::concepts::ambient::core::*;
}
