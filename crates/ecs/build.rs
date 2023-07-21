use std::path::Path;

fn main() {
    let header = r#"
    #![allow(missing_docs)]
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

    "#;

    let api_generated_code = ambient_project_macro_common::generate_code(
        None,
        false,
        true,
        ambient_project_macro_common::Context::Host,
        Some("ambient"),
    )
    .unwrap();

    let api_generated_code = format!("{header}{api_generated_code}");

    let generated_path = Path::new("src").join("generated.rs");
    std::fs::write(&generated_path, api_generated_code.trim()).unwrap();
    std::process::Command::new("rustfmt")
        .arg(generated_path)
        .status()
        .unwrap();
}
