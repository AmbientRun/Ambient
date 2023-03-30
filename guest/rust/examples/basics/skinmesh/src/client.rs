use ambient_api::{
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    entity::{AnimationAction, AnimationController},
    prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::{
    camera::aspect_ratio_from_window, prefab::prefab_from_url, primitives::quad,
};
use ambient_ui_components::{
    prelude::{color, lookat_center, main_scene, scale, translation, Button, FlowColumn},
    FocusRoot, UIExt,
};

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

    entity::set_animation_controller(
        unit_id,
        AnimationController {
            actions: &[AnimationAction {
                clip_url: &asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
                looping: true,
                weight: 1.,
            }],
            apply_base_pose: false,
        },
    );
    App::el(unit_id).spawn_interactive()
}

#[element_component]
fn App(_hooks: &mut Hooks, unit: EntityId) -> Element {
    let anim_button = |name, anim| {
        Button::new(name, move |_| {
            entity::set_animation_controller(
                unit,
                AnimationController {
                    actions: &[AnimationAction {
                        clip_url: &asset::url(anim).unwrap(),
                        looping: true,
                        weight: 1.,
                    }],
                    apply_base_pose: false,
                },
            );
        })
        .el()
    };
    FocusRoot::el([FlowColumn::el([
        anim_button(
            "Robot Hip Hop Dance",
            "assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim",
        ),
        anim_button("Capoeira", "assets/Capoeira.fbx/animations/mixamo.com.anim"),
    ])
    .with_background(vec4(0., 0., 0., 0.9))
    .with_padding_even(10.)])
}
