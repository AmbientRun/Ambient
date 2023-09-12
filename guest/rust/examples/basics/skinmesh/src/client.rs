use ambient_api::{
    animation::{self, AnimationPlayerRef, BindId, BlendNodeRef, PlayClipFromUrlNodeRef, PlayMode},
    core::{
        animation::components::apply_animation_player,
        app::components::name,
        layout::components::space_between_items,
        model::components::model_loaded,
        prefab::components::prefab_from_url,
        primitives::{components::quad, concepts::Sphere},
        rendering::components::color,
        transform::components::{local_to_parent, reset_scale, scale},
    },
    element::{use_effect, use_state},
    prelude::*,
};

use packages::{
    orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional},
    this::assets,
};

#[main]
pub async fn main() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            lookat_target: Some(vec3(0., 0., 1.)),
            camera_angle: Some(vec2(-180f32.to_radians(), 45f32.to_radians())),
            camera_distance: Some(3.0),
        },
    }
    .spawn();

    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .with(name(), "Floor".to_string())
        .spawn();

    let unit_id = Entity::new()
        .with(prefab_from_url(), assets::url("Peasant Man.fbx"))
        .with(name(), "Peasant".to_string())
        .spawn();

    let capoeira =
        PlayClipFromUrlNodeRef::new(assets::url("Capoeira.fbx/animations/mixamo.com.anim"));
    let robot = PlayClipFromUrlNodeRef::new(assets::url(
        "Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
    ));
    let blend = BlendNodeRef::new(capoeira, robot, 0.);
    let anim_player = AnimationPlayerRef::new(blend);
    entity::add_component(unit_id, apply_animation_player(), anim_player.0);

    println!("Robot duration: {} sec", robot.clip_duration().await);

    let _ = entity::wait_for_component(unit_id, model_loaded()).await;

    // This demonstrates how to attach an entity to a bone
    let left_foot = animation::get_bone_by_bind_id(unit_id, &BindId::LeftFoot).unwrap();
    let ball = Entity::new()
        .with_merge(Sphere::suggested())
        .with(scale(), vec3(0.3, 0.3, 0.3))
        .with(color(), vec4(0.0, 1.0, 0.0, 1.0))
        .with(local_to_parent(), Default::default())
        .with(reset_scale(), ())
        .spawn();
    entity::add_child(left_foot, ball);

    let robot = PlayClipFromUrlNodeRef::new(assets::url(
        "Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
    ));
    robot.looping(false);

    App::el(blend, anim_player, robot).spawn_interactive()
}

#[element_component]
fn App(
    hooks: &mut Hooks,
    blend_node: BlendNodeRef,
    anim_player: AnimationPlayerRef,
    robot: PlayClipFromUrlNodeRef,
) -> Element {
    let (blend, set_blend) = use_state(hooks, 0.0f32);
    let (masked, set_masked) = use_state(hooks, false);

    {
        to_owned!(blend_node);
        use_effect(hooks, masked, move |_, &masked| {
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
        use_effect(hooks, blend, move |_, &blend| {
            blend_node.set_weight(blend);
            |_| {}
        });
    }

    FlowColumn::el([
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
                robot.looping(true);
                robot.restart();
                anim_player.play(robot);
            })
            .el(),
            Button::new("Play blend animation", move |_| {
                anim_player.play(blend_node);
            })
            .el(),
            Button::new("Freeze animation", move |_| {
                robot.looping(false);
                robot.set_play_mode(PlayMode::FreezeAtPercentage { percentage: 0.5 });
                anim_player.play(robot);
            })
            .el(),
        ]),
    ])
}
