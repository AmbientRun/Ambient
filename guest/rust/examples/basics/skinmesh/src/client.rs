use ambient_api::{
    animation::{get_bone_by_bind_id, AnimationPlayer, BlendNode, PlayClipFromUrlNode},
    components::core::{
        animation::apply_animation_player, camera::aspect_ratio_from_window, model::model_loaded,
        prefab::prefab_from_url, primitives::quad,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    element::to_owned,
    entity::{add_component, get_component, set_component, wait_for_component},
    messages::Frame,
    prelude::*,
};

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

    wait_for_component(unit_id, model_loaded()).await;

    let left_leg = get_bone_by_bind_id(unit_id, "LeftFoot").unwrap();
    let ball = Entity::new()
        .with_merge(make_transformable())
        .with_merge(make_sphere())
        .with(scale(), vec3(0.3, 0.3, 0.3))
        .with(color(), vec4(0.0, 1.0, 0.0, 1.0))
        .with(translation(), Vec3::ZERO)
        .spawn();
    Frame::subscribe(move |_| {
        let (_, _, pos) = get_component(left_leg, local_to_world())
            .unwrap()
            .to_scale_rotation_translation();
        set_component(ball, translation(), pos);
    });

    App::el(blend, anim_player).spawn_interactive()
}

#[element_component]
fn App(hooks: &mut Hooks, blend_node: BlendNode, anim_player: AnimationPlayer) -> Element {
    let (blend, set_blend) = hooks.use_state(0.0f32);
    let (masked, set_masked) = hooks.use_state(false);

    {
        to_owned!(blend_node);
        hooks.use_effect(masked, move |_, &masked| {
            if masked {
                blend_node.set_mask_humanoid_lower_body(0.);
            } else {
                blend_node.set_mask(vec![]);
            }
            |_| {}
        });
    }

    {
        to_owned!(blend_node);
        hooks.use_effect(blend, move |_, &blend| {
            blend_node.set_weight(blend);
            |_| {}
        });
    }

    FocusRoot::el([FlowColumn::el([
        FlowRow::el([
            Text::el("Capeira"),
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
            Text::el("Robot"),
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
                anim_player.play(robot);
            })
            .el(),
            Button::new("Play blend animation", move |_| {
                anim_player.play(blend_node.clone());
            })
            .el(),
            Button::new("Freeze animation", move |_| {
                let robot = PlayClipFromUrlNode::new(
                    asset::url("assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim")
                        .unwrap(),
                    false,
                );
                robot.freeze_at_percentage(0.5);
                anim_player.play(robot);
            })
            .el(),
        ]),
    ])])
}
