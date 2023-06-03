use ambient_api::{
    animation::{AnimationGraph, AnimationNode},
    components::core::{
        animation::apply_animation_graph, camera::aspect_ratio_from_window,
        prefab::prefab_from_url, primitives::quad,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    entity::{add_component, AnimationAction, AnimationController},
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

    let anim_graph = AnimationGraph::new(AnimationNode::new_play_clip_from_url(
        asset::url(START.1).unwrap(),
        true,
    ));
    add_component(unit_id, apply_animation_graph(), anim_graph.0);

    // entity::set_animation_controller(
    //     unit_id,
    //     AnimationController {
    //         actions: &[
    //             AnimationAction {
    //                 clip_url: &asset::url(START.1).unwrap(),
    //                 looping: true,
    //                 weight: 1.,
    //             },
    //             AnimationAction {
    //                 clip_url: &asset::url(END.1).unwrap(),
    //                 looping: true,
    //                 weight: 0.,
    //             },
    //         ],
    //         apply_base_pose: false,
    //     },
    // );

    // entity::set_animation_action_stack(unit_id, &[entity::AnimationActionStack::Sample(0)]);
    // // Required for animation blend stack
    // entity::set_animation_binder_mask(unit_id, &SKELETON);
    // entity::set_animation_binder_weights(unit_id, LOWER_BODY_MASK_INDEX, &LOWER_BODY_MASK);

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
    let (weight, set_weight) = hooks.use_state(0.0f32);
    let (time, set_time) = hooks.use_state(0.0f32);

    hooks.use_effect((blend, weight, time), move |_, &(w, i, t)| {
        use entity::AnimationActionStack::*;

        let s0 = if t == 0.0 {
            Sample(0)
        } else {
            let time_absolute = durations[0] * t;
            SampleAbsolute(entity::AnimationSampleAbsolute {
                action_index: 0,
                time_absolute,
            })
        };

        // Alternatively SamplePercentage
        let s1 = if t == 0.0 {
            Sample(1)
        } else {
            SamplePercentage(entity::AnimationSamplePercentage {
                action_index: 1,
                time_percentage: t,
            })
        };

        if w != 0.0 {
            entity::set_animation_action_stack(
                unit,
                &[
                    s0,
                    s1,
                    Blend(entity::AnimationStackBlend {
                        weight: w,
                        mask: LOWER_BODY_MASK_INDEX,
                    }),
                ],
            );
        } else {
            entity::set_animation_action_stack(unit, &[s0, s1, Interpolate(i)]);
        }
        |_| {}
    });

    FocusRoot::el([FlowColumn::el([
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
    ])])
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
