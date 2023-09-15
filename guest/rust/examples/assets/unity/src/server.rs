use ambient_api::{
    core::{
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        prefab::components::prefab_from_url,
        primitives::components::quad,
        rendering::components::cast_shadows,
        transform::components::{local_to_world, lookat_target, scale},
    },
    prelude::*,
};

use packages::this::assets;

#[main]
pub fn main() {
    PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            aspect_ratio_from_window: Some(entity::resources()),
            translation: Some(Vec3::ONE * 5. + Vec3::Z * 1.5),
            main_scene: Some(()),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(lookat_target(), Vec3::Z * 1.5)
    .spawn();

    Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 100.)
        .spawn();

    Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(cast_shadows(), ())
        .with(
            prefab_from_url(),
            assets::url("TreePack/Prefabs/Standard/Fir_01_Plant.prefab"),
        )
        .spawn();
}
