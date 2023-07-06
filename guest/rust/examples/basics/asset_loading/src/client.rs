use ambient_api::prelude::*;

#[main]
fn main() {
    // Load the asset
    println!(
        "asset url can be accessed from client: {}",
        // Should this panic?
        asset::url("assets/ube.glb").unwrap()
    );
}
