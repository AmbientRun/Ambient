wit_bindgen_host_wasmtime_rust::export!("../guest/rust/src/internal/host.wit");

use std::{collections::HashMap, path::PathBuf};

pub const SCRIPTING_INTERFACE_NAME: &str = "elements_runtime_scripting_interface";

pub fn get_scripting_interfaces() -> HashMap<String, Vec<(PathBuf, String)>> {
    let mut interfaces = elements_scripting_host::shared::interface::get_scripting_interfaces();
    interfaces.insert(
        SCRIPTING_INTERFACE_NAME.to_string(),
        get_scripting_interface(),
    );
    interfaces
}

fn get_scripting_interface() -> Vec<(PathBuf, String)> {
    let interface_json = include_str!(concat!(
        env!("OUT_DIR"),
        "/elements_runtime_scripting_interface.json"
    ));
    serde_json::from_str(interface_json).unwrap()
}
