use ambient_api::prelude::*;

mod common;

pub mod packages;

#[main]
pub async fn main() {
    common::main().await
}
