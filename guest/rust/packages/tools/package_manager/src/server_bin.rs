use ambient_api::prelude::*;

mod shared;

mod server;
use server::*;

#[main]
pub async fn main() {
    package_load::main();
    package_view::main();
    package_manager::main();
}
