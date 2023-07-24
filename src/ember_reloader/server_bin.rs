use ambient_api::prelude::*;

use shared::*;
mod shared;

mod server;
use server::*;

#[main]
pub async fn main() {
    ember_load::main();
    wasm_manager::main();
}
