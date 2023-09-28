use ambient_api::{
    core::{
        app::components::name,
        camera::{
            components::fog,
            concepts::{
                PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
            },
        },
        transform::components::local_to_parent,
    },
    prelude::*,
};
use packages::{
    character_animation::components as cac, tangent_schema::character::components as cc,
    unit_schema::components as uc,
};

#[main]
pub fn main() {
    spawn_query((cc::is_character(), cc::player_ref(), uc::head_ref())).bind(move |characters| {
        for (character_id, (_, player_ref, head)) in characters {
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
            .with(fog(), ())
            .spawn();

            entity::add_child(head, camera);
            entity::add_component(
                head,
                packages::tangent_schema::character::head::components::camera_ref(),
                camera,
            );
            entity::add_components(
                character_id,
                Entity::new()
                    .with(
                        cac::idle(),
                        packages::this::assets::url("Idle.fbx/animations/mixamo.com.anim"),
                    )
                    .with(
                        cac::jump(),
                        packages::this::assets::url("Jump.fbx/animations/mixamo.com.anim"),
                    )
                    .with(
                        cac::walk_forward(),
                        packages::this::assets::url("Walking.fbx/animations/mixamo.com.anim"),
                    )
                    .with(
                        cac::walk_backward(),
                        packages::this::assets::url(
                            "Walking_Backward.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::walk_left(),
                        packages::this::assets::url(
                            "Left_Strafe_Walk.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::walk_right(),
                        packages::this::assets::url(
                            "Right_Strafe_Walking.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::walk_forward_left(),
                        packages::this::assets::url(
                            "Left_Strafe_Walk.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::walk_forward_right(),
                        packages::this::assets::url(
                            "Right_Strafe_Walking.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::walk_backward_left(),
                        packages::this::assets::url(
                            "Left_Strafe_Walk.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::walk_backward_right(),
                        packages::this::assets::url(
                            "Right_Strafe_Walking.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::run_forward(),
                        packages::this::assets::url("Running_1.fbx/animations/mixamo.com.anim"),
                    )
                    .with(
                        cac::run_left(),
                        packages::this::assets::url("Left_Strafe.fbx/animations/mixamo.com.anim"),
                    )
                    .with(
                        cac::run_right(),
                        packages::this::assets::url("Right_Strafe.fbx/animations/mixamo.com.anim"),
                    )
                    .with(
                        cac::run_forward_left(),
                        packages::this::assets::url(
                            "Jog_Forward_Diagonal_left.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::run_forward_right(),
                        packages::this::assets::url(
                            "Jog_Forward_Diagonal_right.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::run_backward_left(),
                        packages::this::assets::url(
                            "Jog_Backward_Diagonal_left.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::run_backward_right(),
                        packages::this::assets::url(
                            "Jog_Backward_Diagonal_right.fbx/animations/mixamo.com.anim",
                        ),
                    )
                    .with(
                        cac::run_backward(),
                        packages::this::assets::url(
                            "Running_Backward.fbx/animations/mixamo.com.anim",
                        ),
                    ),
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
