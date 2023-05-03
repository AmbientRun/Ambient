use ambient_api::{
    components::core::{
        camera::aspect_ratio_from_window, prefab::prefab_from_url, primitives::quad,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    entity::{AnimationAction, AnimationController},
    prelude::*,
};

const START: (&str, &str) = (
    "Robot Hip Hop Dance",
    "assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
);
const END: (&str, &str) = ("Capoeira", "assets/Capoeira.fbx/animations/mixamo.com.anim");

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
                    weight: 1.,
                },
                AnimationAction {
                    clip_url: &asset::url(END.1).unwrap(),
                    looping: true,
                    weight: 0.,
                },
            ],
            apply_base_pose: false,
        },
    );

    let start = &asset::url(START.1).unwrap();
    let end = &asset::url(END.1).unwrap();
    block_until(move || {
        entity::has_animation_clip(start) &&
        entity::has_animation_clip(end)
    }).await;


    let clips = entity::get_animation_clips(&[&start, &end]);

    App::el(unit_id, [clips[0].duration, clips[1].duration]).spawn_interactive()
}

#[element_component]
fn App(hooks: &mut Hooks, unit: EntityId, durations: [f32; 2]) -> Element {
    let (weight, set_weight) = hooks.use_state(0.0f32);
    hooks.use_effect(weight, move |_, t| {
        entity::set_animation_blend(unit, &[1. - *t, *t], &[], false);
        |_| {}
    });


    let (time, set_time) = hooks.use_state(0.0f32);
    hooks.use_effect(time, move |_, t| {
        if *t != 0.0 {
            let absolute_time = [durations[0] * t, durations[1] * t];
            entity::set_animation_blend(unit, &[], &absolute_time, true);
            // Alternatively: entity::set_animation_blend(unit, &[], &[*t, *t], false);
        }
        |_| {}
    });

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
        ])
    ])
}
