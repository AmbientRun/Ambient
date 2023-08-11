use ambient_api::prelude::*;

#[main]
fn main() {
    // Load the asset
    println!(
        "asset url can be accessed from client: {}",
        embers::ambient_example_asset_loading::assets::url("Cube.glb")
    );
}
