use ambient_api::{
    components::core::{
        camera::aspect_ratio_from_window, prefab::prefab_from_url, primitives::quad,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    entity::{AnimationAction, AnimationController},
    prelude::*,
};

const END: (&str, &str) = (
    "Robot Hip Hop Dance",
    "assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
);
const START: (&str, &str) = ("Capoeira", "assets/Capoeira.fbx/animations/mixamo.com.anim");

#[main]
pub async fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(2., 2., 3.0))
        .with(lookat_target(), vec3(0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .with(name(), "Floor".to_string())
        .spawn();

    let unit_id = Entity::new()
        .with_merge(make_transformable())
        .with(
            prefab_from_url(),
            asset::url("assets/Peasant Man.fbx").unwrap(),
        )
        .with(name(), "Peasant".to_string())
        .spawn();

    entity::set_animation_controller(
        unit_id,
        AnimationController {
            actions: &[
                AnimationAction {
                    clip_url: &asset::url(START.1).unwrap(),
                    looping: true,
                },
                AnimationAction {
                    clip_url: &asset::url(END.1).unwrap(),
                    looping: true,
                },
            ],
            apply_base_pose: false,
        },
    );

    // Required for animation blend operation
    entity::set_animation_binder_mask(unit_id, &SKELETON);
    entity::set_animation_binder_weights(unit_id, LOWER_BODY_MASK_INDEX, &LOWER_BODY_MASK);

    let start_url = asset::url(START.1).unwrap();
    let end_url = asset::url(END.1).unwrap();
    let assets: &[&str] = &[&start_url, &end_url];

    asset::block_until_animations_are_loaded(assets).await;
    let clips = asset::get_animation_asset_metadata(assets);

    App::el(unit_id, [clips[0].duration, clips[1].duration]).spawn_interactive()
}



#[element_component]
fn App(hooks: &mut Hooks, unit: EntityId, durations: [f32; 2]) -> Element {
    let (blend, set_blend) = hooks.use_state(0.0f32);
    let (weight, set_weight) = hooks.use_state(1.0f32);
    let (time, set_time) = hooks.use_state(0.0f32);
    let (duration, set_duration) = hooks.use_state(0.2f32);

    let (speed, set_speed) = hooks.use_state(1.0f32);

    hooks.use_effect((blend, weight, time, speed), move |_, &(w, i, t, s)| {
            use entity::AnimationActionStack::*;
            const FIRST_ANIMATION: u32 = 0;
            const SECOND_ANIMATION: u32 = 1;

            if w == 0.0 && t == 0.0 {
                if i == 0.0 {
                    entity::play_animation_action_index(unit, FIRST_ANIMATION, s, duration);
                } else if i == 1.0 {
                    entity::play_animation_action_index(unit, SECOND_ANIMATION, s, duration);
                } else {
                    entity::set_animation_action_stack(unit, &[Sample(FIRST_ANIMATION), Sample(SECOND_ANIMATION), Interpolate(i)], duration);
                }
            } else {
                let s0 = if t == 0.0 {
                    Sample(FIRST_ANIMATION)
                } else {
                    let time_absolute = durations[0] * t;
                    SampleAbsolute(entity::AnimationSampleAbsolute {
                        action_index: FIRST_ANIMATION,
                        time_absolute,
                    })
                };

                // Alternatively SamplePercentage
                let s1 = if t == 0.0 {
                    Sample(SECOND_ANIMATION)
                } else {
                    SamplePercentage(entity::AnimationSamplePercentage {
                        action_index: SECOND_ANIMATION,
                        time_percentage: t,
                    })
                };

                if w != 0.0 {
                    entity::set_animation_action_stack(unit, &[s0, s1, Blend(entity::AnimationStackBlend {
                        weight: w,
                        mask: LOWER_BODY_MASK_INDEX,
                    })], duration);
                } else {
                    entity::set_animation_action_stack(unit, &[s0, s1, Interpolate(i)], duration);
                }
            }

        |_| {}
    });


    let set_animation_weight = set_weight.clone();
    let set_animation_time = set_time.clone();
    let set_animation_blend = set_blend.clone();

    FocusRoot::el([
        FlowColumn::el([
            FlowRow::el([
                Text::el(START.0),
                Slider {
                    value: weight,
                    on_change: Some(cb(move |weight| {
                        set_weight(weight);
                    })),
                    min: 0.,
                    max: 1.,
                    width: 100.,
                    logarithmic: false,
                    round: Some(2),
                    suffix: None,
                }
                .el(),
                Text::el(END.0),
                Text::el(" (interpolate)"),
            ])
            .with(space_between_items(), 4.0)
            .with_background(vec4(0., 0., 0., 0.9))
            .with_padding_even(10.),
            FlowRow::el([
                Text::el(START.0),
                Slider {
                    value: blend,
                    on_change: Some(cb(move |blend| {
                        set_blend(blend);
                    })),
                    min: 0.,
                    max: 1.,
                    width: 100.,
                    logarithmic: false,
                    round: Some(2),
                    suffix: None,
                }
                .el(),
                Text::el(END.0),
                Text::el(" (blend lower body)"),
            ])
            .with(space_between_items(), 4.0)
            .with_background(vec4(0., 0., 0., 0.9))
            .with_padding_even(10.),
            FlowRow::el([
                Text::el("Time"),
                Slider {
                    value: time,
                    on_change: Some(cb(move |time| {
                        set_time(time);
                    })),
                    min: 0.,
                    max: 1.,
                    width: 100.,
                    logarithmic: false,
                    round: Some(2),
                    suffix: None,
                }
                .el(),
            ])
            .with(space_between_items(), 4.0)
            .with_background(vec4(0., 0., 0., 0.9))
            .with_padding_even(10.),
            FlowRow::el([
                Text::el("Speed"),
                Slider {
                    value: speed,
                    on_change: Some(cb(move |speed| {
                        set_speed(speed);
                    })),
                    min: 0.1,
                    max: 5.0,
                    width: 100.,
                    logarithmic: false,
                    round: Some(2),
                    suffix: None,
                }
                .el(),
            ])
            .with(space_between_items(), 4.0)
            .with_background(vec4(0., 0., 0., 0.9))
            .with_padding_even(10.),
            FlowRow::el([
                Text::el("Transition Duration"),
                Slider {
                    value: duration,
                    on_change: Some(cb(move |duration| {
                        set_duration(duration);
                    })),
                    min: 0.1,
                    max: 2.,
                    width: 100.,
                    logarithmic: false,
                    round: Some(2),
                    suffix: None,
                }
                .el(),
                Button::new("Swap Animation", move |_| {
                    set_animation_time(0.0);
                    set_animation_blend(0.0);

                    if weight > 0.0 {
                        set_animation_weight(0.0);
                    } else {
                        set_animation_weight(1.0);
                    }
                })
                .hotkey(VirtualKeyCode::Space)
                .el(),

            ])
            .with(space_between_items(), 4.0)
            .with_background(vec4(0., 0., 0., 0.9))
            .with_padding_even(10.),
        ])
    ])
}

const LOWER_BODY_MASK_INDEX: u32 = 0;
const LOWER_BODY_MASK: [f32; 9] = [1.0; 9];
const SKELETON: [&str; 52] = [
    // Lower body for convenience
    "Hips",
    "LeftFoot",
    "LeftLeg",
    "LeftToeBase",
    "LeftUpLeg",
    "RightFoot",
    "RightLeg",
    "RightToeBase",
    "RightUpLeg",

    // Upper
    "Head",
    "LeftArm",
    "LeftForeArm",
    "LeftHand",
    "LeftHandIndex1",
    "LeftHandIndex2",
    "LeftHandIndex3",
    "LeftHandMiddle1",
    "LeftHandMiddle2",
    "LeftHandMiddle3",
    "LeftHandPinky1",
    "LeftHandPinky2",
    "LeftHandPinky3",
    "LeftHandRing1",
    "LeftHandRing2",
    "LeftHandRing3",
    "LeftHandThumb1",
    "LeftHandThumb2",
    "LeftHandThumb3",
    "LeftShoulder",
    "Neck",
    "RightArm",
    "RightForeArm",
    "RightHand",
    "RightHandIndex1",
    "RightHandIndex2",
    "RightHandIndex3",
    "RightHandMiddle1",
    "RightHandMiddle2",
    "RightHandMiddle3",
    "RightHandPinky1",
    "RightHandPinky2",
    "RightHandPinky3",
    "RightHandRing1",
    "RightHandRing2",
    "RightHandRing3",
    "RightHandThumb1",
    "RightHandThumb2",
    "RightHandThumb3",
    "RightShoulder",
    "Spine",
    "Spine1",
    "Spine2",
];