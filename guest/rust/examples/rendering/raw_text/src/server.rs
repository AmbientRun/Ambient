use ambient_api::{
    core::{
        app::components::main_scene,
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        rendering::components::color,
        text::components::text,
        transform::components::{
            local_to_world, lookat_target, mesh_to_local, mesh_to_world, scale, translation,
        },
    },
    prelude::*,
};

#[main]
pub fn main() {
    PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            translation: Some(vec3(5., 5., 4.)),
            main_scene: Some(()),
            aspect_ratio_from_window: Some(entity::resources()),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(lookat_target(), vec3(0., 0., 0.))
    .spawn();

    Entity::new()
        .with(text(), "Hello world".to_string())
        .with(color(), vec4(1., 1., 1., 1.))
        .with(translation(), vec3(0., 0., 0.01))
        .with(scale(), Vec3::ONE * 0.05)
        .with(local_to_world(), Default::default())
        .with(mesh_to_local(), Default::default())
        .with(mesh_to_world(), Default::default())
        .with(main_scene(), ())
        .spawn();
}
