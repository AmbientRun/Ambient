use ambient_api::{
    animation::{AnimationPlayer, BlendNode, PlayClipFromUrlNode},
    components::core::{
        animation::apply_animation_player, camera::aspect_ratio_from_window,
        prefab::prefab_from_url, primitives::quad,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    element::to_owned,
    entity::add_component,
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

    let capoeira = PlayClipFromUrlNode::new(
        asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
        true,
    );
    let robot = PlayClipFromUrlNode::new(
        asset::url("assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim").unwrap(),
        true,
    );
    let blend = BlendNode::new(&capoeira, &robot, 0.);
    let anim_player = AnimationPlayer::new(&blend);
    add_component(unit_id, apply_animation_player(), anim_player.0);

    println!("Robot duration: {} sec", robot.clip_duration().await);

    let start_url = asset::url(START.1).unwrap();
    let end_url = asset::url(END.1).unwrap();
    let assets: &[&str] = &[&start_url, &end_url];

    asset::block_until_animations_are_loaded(assets).await;
    let clips = asset::get_animation_asset_metadata(assets);

    App::el(blend, anim_player).spawn_interactive()
}

#[element_component]
fn App(hooks: &mut Hooks, blend_node: BlendNode, anim_player: AnimationPlayer) -> Element {
    let (blend, set_blend) = hooks.use_state(0.0f32);
    let (masked, set_masked) = hooks.use_state(false);

    {
        to_owned!(blend_node);
        hooks.use_effect((masked), move |_, &(masked)| {
            if masked {
                blend_node.set_mask(vec![
                    ("Hips".to_string(), 0.),
                    ("LeftFoot".to_string(), 0.),
                    ("LeftLeg".to_string(), 0.),
                    ("LeftToeBase".to_string(), 0.),
                    ("LeftUpLeg".to_string(), 0.),
                    ("RightFoot".to_string(), 0.),
                    ("RightLeg".to_string(), 0.),
                    ("RightToeBase".to_string(), 0.),
                    ("RightUpLeg".to_string(), 0.),
                ]);
            } else {
                blend_node.set_mask(vec![]);
            }
            |_| {}
        });
    }

    {
        to_owned!(blend_node);
        hooks.use_effect((blend), move |_, &(blend)| {
            blend_node.set_weight(blend);
            |_| {}
        });
    }

    FocusRoot::el([FlowColumn::el([
        FlowRow::el([
            Text::el(START.0),
            Slider {
                value: blend,
                on_change: Some(set_blend),
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
            Text::el("Masked"),
            Checkbox {
                value: masked,
                on_change: set_masked,
            }
            .el(),
        ]),
        FlowRow::el([
            Button::new("Play single animation", move |_| {
                let robot = PlayClipFromUrlNode::new(
                    asset::url("assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim")
                        .unwrap(),
                    false,
                );
                anim_player.set_root(robot);
            })
            .el(),
            Button::new("Play blend animation", move |_| {
                anim_player.set_root(blend_node.clone());
            })
            .el(),
            Button::new("Freeze animation", move |_| {
                let robot = PlayClipFromUrlNode::new(
                    asset::url("assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim")
                        .unwrap(),
                    false,
                );
                robot.freeze_at_percentage(0.5);
                anim_player.set_root(robot);
            })
            .el(),
        ]),
    ])])
}

const LOWER_BODY_MASK_IDS: [&str; 9] = [
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
];

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
