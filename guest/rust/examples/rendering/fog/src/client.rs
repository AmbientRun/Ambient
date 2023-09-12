use ambient_api::{
    core::{
        app::components::main_scene,
        camera::components::fog,
        primitives::components::{cube, quad},
        rendering::components::{
            cast_shadows, color, fog_color, fog_density, fog_height_falloff, light_diffuse, sky,
            sun,
        },
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};
use packages::orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional};

#[main]
fn main() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            camera_angle: Some(vec2(0.0, 15f32.to_radians())),
            lookat_target: Some(vec3(0., 0., 2.)),
            ..default()
        },
    }
    .make()
    .with(fog(), ())
    .spawn();

    let sun = Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-1.))
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_color(), vec3(1., 1., 1.))
        .with(fog_density(), 0.1)
        .with(fog_height_falloff(), 0.01)
        .spawn();

    Entity::new().with(sky(), ()).spawn();

    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 1000.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    for i in 0..10 {
        Entity::new()
            .with(cube(), ())
            .with(translation(), vec3(0., 1. * (2f32).powi(i), 1.))
            .with(scale(), Vec3::ONE * 2.)
            .with(color(), vec4(0., 1., 0., 1.))
            .with(cast_shadows(), ())
            .spawn();
    }

    App::el(sun).spawn_interactive();
}

#[element_component]
fn App(hooks: &mut Hooks, sun: EntityId) -> Element {
    FlowColumn::el([
        FlowRow::el([
            Text::el("Fog density: "),
            Slider::new_for_entity_component(hooks, sun, fog_density()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog height falloff: "),
            Slider::new_for_entity_component(hooks, sun, fog_height_falloff()).el(),
        ]),
    ])
    .with_background(vec4(0., 0., 0., 0.9))
    .with_padding_even(10.)
}
