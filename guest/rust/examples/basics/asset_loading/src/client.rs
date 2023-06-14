use ambient_api::{
    components::core::camera::{aspect_ratio_from_window, fog},
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
fn main() {
    // Load the asset
    println!(
        "asset url can be accessed from client: {}",
        asset::url("assets/Cube.glb").unwrap()
    );
}
