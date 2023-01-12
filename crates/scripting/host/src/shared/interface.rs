use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub use host::*;

use super::write_files_to_directory;

wit_bindgen_host_wasmtime_rust::export!("../guest/rust/src/internal/host.wit");
wit_bindgen_host_wasmtime_rust::import!("../guest/rust/src/internal/guest.wit");

pub mod shared {
    // extremely bad no good hack necessary because of https://github.com/bytecodealliance/wit-bindgen/issues/293
    include!("../../../guest/rust/src/internal/shared.rs");
}

pub fn get_scripting_interface() -> Vec<(PathBuf, String)> {
    let interface_json = include_str!(concat!(
        env!("OUT_DIR"),
        "/elements_guest_rust_interface.json"
    ));
    serde_json::from_str(interface_json).unwrap()
}

pub fn write_scripting_interfaces(
    scripting_interfaces: &HashMap<String, Vec<(PathBuf, String)>>,
    interface_root_path: &Path,
) -> anyhow::Result<()> {
    for (interface_name, interface) in scripting_interfaces {
        let interface_path = interface_root_path.join(interface_name);
        let _ = std::fs::remove_dir_all(interface_path.join("src"));
        write_files_to_directory(&interface_path, interface)?;
    }
    Ok(())
}
