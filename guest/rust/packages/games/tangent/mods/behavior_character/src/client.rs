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
use packages::{
    tangent_schema::player::character::components as pcc, unit_schema::components as uc,
};

#[main]
pub fn main() {
    spawn_query((pcc::is_character(), pcc::player_ref(), uc::head_ref())).bind(move |characters| {
        for (_, (_, player_ref, head)) in characters {
            if player_ref != player::get_local() {
                continue;
            }

            let camera = PerspectiveInfiniteReverseCamera {
                optional: PerspectiveInfiniteReverseCameraOptional {
                    translation: Some(vec3(1.0, 0.0, -1.5)),
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
        }
    });
}
