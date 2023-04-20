use ambient_api::{
    components::core::{
        camera::aspect_ratio_from_window, prefab::prefab_from_url, primitives::quad,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    entity::{AnimationAction, AnimationController},
    prelude::*,
};
use ambient_ui::prelude::*;

#[main]
pub fn main() {
    Entity::new()
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

    App::el(unit_id).spawn_interactive()
}

#[element_component]
fn App(hooks: &mut Hooks, unit: EntityId) -> Element {
    const START: (&str, &str) = (
        "Robot Hip Hop Dance",
        "assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
    );
    const END: (&str, &str) = ("Capoeira", "assets/Capoeira.fbx/animations/mixamo.com.anim");

    let (weight, set_weight) = hooks.use_state(0.0f32);
    hooks.use_effect(weight, move |_, weight| {
        entity::set_animation_controller(
            unit,
            AnimationController {
                actions: &[
                    AnimationAction {
                        clip_url: &asset::url(START.1).unwrap(),
                        looping: true,
                        weight: 1. - *weight,
                    },
                    AnimationAction {
                        clip_url: &asset::url(END.1).unwrap(),
                        looping: true,
                        weight: *weight,
                    },
                ],
                apply_base_pose: false,
            },
        );

        |_| {}
    });

    FocusRoot::el([FlowRow::el([
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
    ])
    .with(space_between_items(), 4.0)
    .with_background(vec4(0., 0., 0., 0.9))
    .with_padding_even(10.)])
}
