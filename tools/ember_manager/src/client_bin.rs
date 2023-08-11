use ambient_api::prelude::*;

mod shared;

mod client;
use client::{ember_load::EmberLoad, ember_manager::EmberManager, wasm_manager::WasmManager};

#[main]
pub fn main() {
    App {}.el().spawn_interactive();
}

#[element_component]
pub fn App(_hooks: &mut Hooks) -> Element {
    Group::el([EmberLoad::el(), WasmManager::el(), EmberManager::el()])
}
