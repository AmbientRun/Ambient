use ambient_api::{
    components::core::{
        app::main_scene,
        camera::{aspect_ratio_from_window, fog},
        primitives::{cube, quad},
        rendering::{
            cast_shadows, color, fog_color, fog_density, fog_height_falloff, light_diffuse, sky,
            sun,
        },
        transform::{lookat_target, rotation, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with_default(fog())
        .with(translation(), vec3(0., -5., 3.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .spawn();

    let sun = Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with(rotation(), Quat::from_rotation_y(-1.))
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_color(), vec3(1., 1., 1.))
        .with(fog_density(), 0.1)
        .with(fog_height_falloff(), 0.01)
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 1000.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    for i in 0..10 {
        Entity::new()
            .with_merge(make_transformable())
            .with_default(cube())
            .with(translation(), vec3(0., 1. * (2f32).powi(i), 1.))
            .with(scale(), Vec3::ONE * 2.)
            .with(color(), vec4(0., 1., 0., 1.))
            .with_default(cast_shadows())
            .spawn();
    }

    App::el(sun).spawn_interactive();
}

#[element_component]
fn App(hooks: &mut Hooks, sun: EntityId) -> Element {
    FocusRoot::el([FlowColumn::el([
        FlowRow::el([
            Text::el("Fog density: "),
            Slider::new_for_entity_component(hooks, sun, fog_density()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog height falloff: "),
            Slider::new_for_entity_component(hooks, sun, fog_height_falloff()).el(),
        ]),
        Button::new(
            "Stop audio",
            move |_| {
                println!("you clicked the button!")
            },
        )
        .el()
        .with_background(vec4(0.0, 0.5, 0.9, 1.0)),
    ])
    .with_background(vec4(0., 0., 0., 0.9))
    .with_padding_even(10.)])
}
