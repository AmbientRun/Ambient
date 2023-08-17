use ambient_api::prelude::*;

mod shared;

mod client;
use client::{
    package_load::PackageLoad, package_manager::PackageManager, wasm_manager::WasmManager,
};

#[main]
pub fn main() {
    App {}.el().spawn_interactive();
}

#[element_component]
pub fn App(_hooks: &mut Hooks) -> Element {
    Group::el([PackageLoad::el(), WasmManager::el(), PackageManager::el()])
}
