use ambient_api::prelude::*;

#[main]
fn main() {
    // Load the asset
    println!("asset url can be accessed from client: {}", asset::url("assets/Cube.glb").unwrap());
}