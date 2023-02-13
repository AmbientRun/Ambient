use kiwi_api::{
    components::core::{
        app::main_scene,
        camera::{active_camera, aspect_ratio_from_window, perspective_infinite_reverse},
        object::object_from_url,
        transform::{lookat_center, translation},
    },
    entity::{AnimationAction, AnimationController},
    player::KeyCode,
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    entity::game_object_base()
        .with_default(main_scene())
        .with(active_camera(), 0.)
        .with(translation(), vec3(5.0, 5.0, 4.0))
        .with(lookat_center(), vec3(0., 0., 0.))
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio_from_window(), ())
        .spawn(false);

    let unit_uid = entity::game_object_base()
        .with(object_from_url(), "assets/Peasant Man.fbx".to_string())
        .spawn(false);
    let unit_entity = entity::wait_for_spawn(&unit_uid).await;

    entity::set_animation_controller(
        unit_entity,
        AnimationController {
            actions: &[AnimationAction {
                clip_url: "assets/Capoeira.fbx/animations/mixamo.com.anim",
                looping: true,
                weight: 1.,
            }],
            apply_base_pose: false,
        },
    );

    on(event::FRAME, move |_| {
        for player in player::get_all() {
            let Some((delta, _)) = player::get_raw_input_delta(player) else { continue; };

            if delta.keys.contains(&KeyCode::Key1) {
                entity::set_animation_controller(
                    unit_entity,
                    AnimationController {
                        actions: &[AnimationAction {
                            clip_url: "assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
                            looping: true,
                            weight: 1.,
                        }],
                        apply_base_pose: false,
                    },
                );
            }

            if delta.keys.contains(&KeyCode::Key2) {
                entity::set_animation_controller(
                    unit_entity,
                    AnimationController {
                        actions: &[AnimationAction {
                            clip_url: "assets/Capoeira.fbx/animations/mixamo.com.anim",
                            looping: true,
                            weight: 1.,
                        }],
                        apply_base_pose: false,
                    },
                );
            }

            if delta.keys.contains(&KeyCode::Key3) {
                entity::set_animation_controller(
                    unit_entity,
                    AnimationController {
                        actions: &[
                            AnimationAction {
                                clip_url:
                                    "assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
                                looping: true,
                                weight: 0.5,
                            },
                            AnimationAction {
                                clip_url: "assets/Capoeira.fbx/animations/mixamo.com.anim",
                                looping: true,
                                weight: 0.5,
                            },
                        ],
                        apply_base_pose: false,
                    },
                );
            }
        }
        EventOk
    });

    EventOk
}
