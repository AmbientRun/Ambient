use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window,
            concepts::{
                PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
            },
        },
        primitives::components::quad,
        rendering::components::pbr_material_from_url,
        transform::components::{lookat_target, scale, translation},
    },
    prelude::*,
};

#[main]
pub fn main() {
    PerspectiveInfiniteReverseCamera {
        local_to_world: Mat4::IDENTITY,
        near: 0.1,
        projection: Mat4::IDENTITY,
        projection_view: Mat4::IDENTITY,
        active_camera: 0.0,
        inv_local_to_world: Mat4::IDENTITY,
        fovy: 1.0,
        aspect_ratio: 1.0,
        perspective_infinite_reverse: (),
        optional: PerspectiveInfiniteReverseCameraOptional {
            translation: Some(vec3(5., 5., 6.)),
            main_scene: Some(()),
            aspect_ratio_from_window: Some(entity::resources()),
            ..default()
        },
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
