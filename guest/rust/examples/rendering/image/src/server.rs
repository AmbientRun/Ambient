use ambient_api::{
    core::{
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        primitives::components::quad,
        rendering::components::pbr_material_from_url,
        transform::components::{lookat_target, scale},
    },
    prelude::*,
};

#[main]
pub fn main() {
    PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            translation: Some(vec3(5., 5., 6.)),
            main_scene: Some(()),
            aspect_ratio_from_window: Some(entity::resources()),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(lookat_target(), vec3(0., 0., 2.))
    .spawn();

    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.)
        .with(
            pbr_material_from_url(),
            packages::this::assets::url("pipeline.toml/0/mat.json"),
        )
        .spawn();
}
