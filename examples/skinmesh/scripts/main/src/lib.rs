use components::core::{
    app::main_scene,
    camera::{
        active_camera, aspect_ratio, aspect_ratio_from_window, fovy, near, perspective_infinite_reverse, projection, projection_view,
    },
    ecs::dont_store,
    transform::{inv_local_to_world, local_to_world, lookat_center, lookat_up, rotation, translation},
};
use tilt_runtime_scripting_interface::{
    entity::{AnimationAction, AnimationController},
    player::KeyCode,
    *,
};

#[main]
pub async fn main() -> EventResult {
    entity::game_object_base()
        .with_default(main_scene())
        .with(active_camera(), 0.)
        .with_default(dont_store())
        .with(translation(), vec3(5.0, 5.0, 9.0))
        .with_default(rotation())
        .with(lookat_up(), vec3(0., 0., 1.))
        .with(lookat_center(), vec3(0., 0., 0.))
        .with_default(local_to_world())
        .with_default(inv_local_to_world())
        .with(near(), 0.1)
        .with(fovy(), 1.0)
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio(), 1.)
        .with(aspect_ratio_from_window(), ())
        .with_default(projection())
        .with_default(projection_view())
        .spawn(false);

    let unit_ref = ObjectRef::new("assets/Peasant Man.fbx/objects/main.json");
    let unit_uid = entity::spawn_template(&unit_ref, Vec3::new(0.0, 0.0, 1.0), None, None, false);
    let unit_entity = entity::wait_for_spawn(&unit_uid).await;

    entity::set_animation_controller(
        unit_entity,
        AnimationController {
            actions: &[AnimationAction { clip_url: "assets/Capoeira.fbx/animations/mixamo.com.anim", looping: true, weight: 1. }],
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
        }
        EventOk
    });

    EventOk
}
