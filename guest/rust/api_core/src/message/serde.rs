pub use ambient_package_rt::message_serde::*;

use ambient_shared_types::procedural_storage_handle_definitions;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use paste::paste;

use crate::global::{
    EntityId, ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
    ProceduralTextureHandle,
};

impl MessageSerde for EntityId {
    fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
        output.write_u128::<BigEndian>(self.0)?;
        Ok(())
    }

    fn deserialize_message_part(input: &mut dyn std::io::Read) -> Result<Self, MessageSerdeError> {
        Ok(Self(input.read_u128::<BigEndian>()?))
    }
}

// TODO: check that this is consistent with the host definition
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
