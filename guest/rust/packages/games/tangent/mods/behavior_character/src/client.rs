use ambient_api::{
    core::{
        app::components::name,
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        transform::components::local_to_parent,
    },
    prelude::*,
};
use packages::{tangent_schema::character::components as cc, unit_schema::components as uc};

#[main]
pub fn main() {
    spawn_query((cc::is_character(), cc::player_ref(), uc::head_ref())).bind(move |characters| {
        for (_, (_, player_ref, head)) in characters {
            if player_ref != player::get_local() {
                continue;
            }

            let camera = PerspectiveInfiniteReverseCamera {
                optional: PerspectiveInfiniteReverseCameraOptional {
                    translation: Some(vec3(1.0, 0.0, -2.5)),
                    main_scene: Some(()),
                    aspect_ratio_from_window: Some(entity::resources()),
                    ..default()
                },
                ..PerspectiveInfiniteReverseCamera::suggested()
            }
            .make()
            .with(local_to_parent(), Default::default())
            .with(name(), "Camera".to_string())
            .spawn();

            entity::add_child(head, camera);
            entity::add_component(
                head,
                packages::tangent_schema::character::head::components::camera_ref(),
                camera,
            );
        }
    });

    despawn_query(packages::tangent_schema::character::head::components::camera_ref()).bind(
        |heads| {
            for (_, camera_id) in heads {
                entity::despawn(camera_id);
            }
        },
    );
}
