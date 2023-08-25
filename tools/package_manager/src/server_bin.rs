use ambient_api::prelude::*;

mod shared;

mod server;
use server::*;

#[main]
pub async fn main() {
    package_load::main();
    wasm_manager::main();
    package_manager::main();
}
