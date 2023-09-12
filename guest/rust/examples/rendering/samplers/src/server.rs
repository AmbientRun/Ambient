use ambient_api::{
    core::{prefab::components::prefab_from_url, transform::components::translation},
    prelude::*,
};
use packages::{orbit_camera::concepts::OrbitCamera, this::assets};

#[main]
pub fn main() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: default(),
    }
    .spawn();

    Entity::new()
        .with(translation(), vec3(-1.25, 0.0, 0.0))
        .with(prefab_from_url(), assets::url("quad-linear.glb"))
        .spawn();

    Entity::new()
        .with(translation(), vec3(1.25, 0.0, 0.0))
        .with(prefab_from_url(), assets::url("quad-nearest.glb"))
        .spawn();
}
