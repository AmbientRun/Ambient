use ambient_api::{
    core::{
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        player::components::is_player,
        primitives::components::cube,
        rendering::components::color,
        transform::components::{lookat_target, translation},
    },
    prelude::*,
};

#[main]
pub fn main() {
    PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            main_scene: Some(()),
            translation: Some(Vec3::ONE * 5.),
            aspect_ratio_from_window: Some(entity::resources()),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(lookat_target(), vec3(0., 0., 0.))
    .spawn();

    spawn_query(is_player()).bind(move |players| {
        // For each player joining, spawn a random colored box somewhere
        for _ in players {
            Entity::new()
                .with(cube(), ())
                .with(translation(), rand::random())
                .with(color(), rand::random::<Vec3>().extend(1.0))
                .spawn();
        }
    });
}
