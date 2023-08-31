use ambient_api::prelude::*;

#[main]
fn main(_world: &mut World) {
    // Load the asset
    println!(
        "asset url can be accessed from client: {}",
        packages::this::assets::url("Cube.glb")
    );
}
