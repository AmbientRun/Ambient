use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../../shared_crates/schema");

    let header = r#"
    #![allow(missing_docs)]
    #![allow(dead_code)]
    #![allow(unused)]
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

    let api_generated_code = ambient_sys::task::make_native_multithreaded_runtime()
        .unwrap()
        .block_on(ambient_project_macro_common::generate_code(
            None,
            ambient_project_macro_common::Context::Host,
            Some("ambient_core"),
        ))
        .unwrap();

    let api_generated_code = format!("{header}{api_generated_code}");

    let generated_path = Path::new("src").join("generated.rs");
    std::fs::write(&generated_path, api_generated_code.trim()).unwrap();
    std::process::Command::new("rustfmt")
        .arg(generated_path)
        .status()
        .unwrap();
}
