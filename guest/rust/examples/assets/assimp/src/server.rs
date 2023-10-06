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

    // Model
    Entity::new()
        .with_merge(Transformable {
            local_to_world: Default::default(),
            optional: Default::default(),
        })
        .with(prefab_from_url(), assets::url("Zombie1.x"))
        .spawn();
}
