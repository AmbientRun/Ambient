use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        player::player,
        prefab::prefab_from_url,
        primitives::quad,
        rendering::color,
        transform::{lookat_center, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    entity::{AnimationAction, AnimationController},
    player::KeyCode,
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    let camera_id = Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(2., 2., 3.0))
        .with(lookat_center(), vec3(0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .spawn();

    let unit_id = Entity::new()
        .with_merge(make_transformable())
        .with(
            prefab_from_url(),
            asset::url("assets/Peasant Man.fbx").unwrap(),
        )
        .spawn();

    entity::set_animation_controller(
        unit_id,
        AnimationController {
            actions: &[AnimationAction {
                clip_url: &asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
                looping: true,
                weight: 1.,
            }],
            apply_base_pose: false,
        },
    );

    query(player()).build().each_frame(move |players| {
        for (player, _) in players {
            let Some((delta, _)) = player::get_raw_input_delta(player) else { continue; };

            let mut camera_position = entity::get_component(camera_id, lookat_center()).unwrap();
            camera_position += vec3(delta.mouse_position.x, 0.0, delta.mouse_position.y) / 1024.0;

            entity::set_component(camera_id, lookat_center(), camera_position);

            if delta.keys.contains(&KeyCode::Key1) {
                entity::set_animation_controller(
                    unit_id,
                    AnimationController {
                        actions: &[AnimationAction {
                            clip_url: &asset::url(
                                "assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
                            )
                            .unwrap(),
                            looping: true,
                            weight: 1.,
                        }],
                        apply_base_pose: false,
                    },
                );
            }

            if delta.keys.contains(&KeyCode::Key2) {
                entity::set_animation_controller(
                    unit_id,
                    AnimationController {
                        actions: &[AnimationAction {
                            clip_url: &asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim")
                                .unwrap(),
                            looping: true,
                            weight: 1.,
                        }],
                        apply_base_pose: false,
                    },
                );
            }

            if delta.keys.contains(&KeyCode::Key3) {
                entity::set_animation_controller(
                    unit_id,
                    AnimationController {
                        actions: &[
                            AnimationAction {
                                clip_url: &asset::url(
                                    "assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
                                )
                                .unwrap(),
                                looping: true,
                                weight: 0.5,
                            },
                            AnimationAction {
                                clip_url: &asset::url(
                                    "assets/Capoeira.fbx/animations/mixamo.com.anim",
                                )
                                .unwrap(),
                                looping: true,
                                weight: 0.5,
                            },
                        ],
                        apply_base_pose: false,
                    },
                );
            }
        }
    });

    EventOk
}
