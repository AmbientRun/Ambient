use ambient_api::{
    core::{
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        prefab::components::prefab_from_url,
        primitives::components::quad,
        transform::{
            components::{lookat_target, translation},
            concepts::Transformable,
        },
    },
    prelude::*,
};
use packages::{orbit_camera::concepts::OrbitCamera, this::assets};

#[main]
pub fn main() {
    // Camera
    OrbitCamera::suggested().spawn();
}
