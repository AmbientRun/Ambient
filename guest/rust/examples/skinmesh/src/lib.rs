use ambient_api::{
    components::core::{
        game_objects::player_camera,
        object::object_from_url,
        player::player,
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
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with_default(player_camera())
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
            object_from_url(),
            asset_url("assets/Peasant Man.fbx").unwrap(),
        )
        .spawn();

    entity::set_animation_controller(
        unit_id,
        AnimationController {
            actions: &[AnimationAction {
                clip_url: &asset_url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
                looping: true,
                weight: 1.,
            }],
            apply_base_pose: false,
        },
    );

    query(player()).build().bind(move |players| {
        for (player, _) in players {
            let Some((delta, _)) = player::get_raw_input_delta(player) else { continue; };

            if delta.keys.contains(&KeyCode::Key1) {
                entity::set_animation_controller(
                    unit_id,
                    AnimationController {
                        actions: &[AnimationAction {
                            clip_url: &asset_url(
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
                            clip_url: &asset_url("assets/Capoeira.fbx/animations/mixamo.com.anim")
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
                                clip_url: &asset_url(
                                    "assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
                                )
                                .unwrap(),
                                looping: true,
                                weight: 0.5,
                            },
                            AnimationAction {
                                clip_url: &asset_url(
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
