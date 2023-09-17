use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../../shared_crates/schema");

    let header = r#"
    #![allow(missing_docs)]
    #![allow(dead_code)]
    #![allow(unused)]
    use std::io::Read;

    use ambient_package_rt::message_serde::{MessageSerde, MessageSerdeError};
    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

    pub use raw::ambient_core::*;

    "#;

    let api_generated_code = ambient_sys::task::make_native_multithreaded_runtime()
        .unwrap()
        .block_on(ambient_package_macro_common::generate_code(
            None,
            ambient_package_macro_common::Context::Host,
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
