use ambient_api::{
    core::{
        app::components::main_scene,
        camera::components::fog,
        model::components::model_from_url,
        primitives::components::{cube, quad},
        rendering::components::{
            cast_shadows, color, fog_color, fog_density, fog_height_falloff, light_ambient,
            light_diffuse, sky, sun,
        },
        transform::components::{lookat_target, rotation, scale, translation},
    },
    prelude::*,
};
use packages::orbit_camera::{
    components::{camera_angle, camera_distance},
    concepts::{OrbitCamera, OrbitCameraOptional},
};
use packages::this::components::{
    amb_b, amb_g, amb_r, fog_b, fog_g, fog_r, orbit_pitch, orbit_target_height, orbit_turn,
    orbit_zoom, sun_b, sun_g, sun_r, sun_rotx, sun_roty, sun_rotz, turntable,
};

#[main]
fn main() {
    let orbitcamera = OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            camera_angle: Some(vec2(0.0, 0.0)),
            lookat_target: Some(vec3(0., 0., 2.)),
            ..default()
        },
    }
    .make()
    .with(fog(), ())
    .with(orbit_turn(), 0.49)
    .with(orbit_zoom(), 0.87)
    .with(orbit_pitch(), 0.34)
    .with(orbit_target_height(), 0.23)
    .spawn();

    query((
        orbit_turn(),
        orbit_zoom(),
        orbit_pitch(),
        orbit_target_height(),
    ))
    .each_frame(|orbitcams| {
        for (cam, (turn, zoom, pitch, targetheight)) in orbitcams {
            entity::add_component(cam, camera_angle(), vec2(6.28 * turn, pitch * 2. - 1.));
            entity::add_component(cam, camera_distance(), (20f32).powf(1. + zoom) * 0.25);
            entity::add_component(cam, lookat_target(), vec3(0., 0., 100.0 * targetheight));
        }
    });

    query(turntable()).each_frame(|turners| {
        for (turner, turn) in turners {
            entity::add_component(turner, rotation(), Quat::from_rotation_z(turn * 6.28));
        }
    });

    let sun = Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-1.))
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_color(), vec3(1., 1., 1.))
        .with(fog_density(), 0.05)
        .with(fog_height_falloff(), 0.1)
        .with(amb_r(), 0.50)
        .with(amb_g(), 0.50)
        .with(amb_b(), 0.50)
        .with(fog_r(), 0.64)
        .with(fog_g(), 0.64)
        .with(fog_b(), 0.64)
        .with(sun_r(), 1.)
        .with(sun_g(), 1.)
        .with(sun_b(), 1.)
        .with(sun_rotx(), 0.02)
        .with(sun_roty(), 0.74)
        .with(sun_rotz(), 0.86)
        .spawn();

    query((amb_r(), amb_g(), amb_b())).each_frame(|suns| {
        for (sun, (ar, ag, ab)) in suns {
            entity::add_component(sun, light_ambient(), vec3(ar, ag, ab));
        }
    });

    query((
        fog_r(),
        fog_g(),
        fog_b(),
        sun_r(),
        sun_g(),
        sun_b(),
        sun_rotx(),
        sun_roty(),
        sun_rotz(),
    ))
    .each_frame(|suns| {
        for (sun, (fr, fg, fb, sr, sg, sb, lx, ly, lz)) in suns {
            entity::add_component(sun, fog_color(), vec3(fr, fg, fb));
            entity::add_component(sun, light_diffuse(), vec3(sr, sg, sb));
            entity::add_component(
                sun,
                rotation(),
                Quat::from_euler(glam::EulerRot::XYZ, lx * 6.28, ly * 6.28, lz * 6.28),
            );
        }
    });

    Entity::new().with(sky(), ()).spawn();

    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 1000.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    for i in 1..10 {
        Entity::new()
            .with(cube(), ())
            .with(translation(), vec3(0., 1. * (2f32).powi(i), 1.))
            .with(scale(), Vec3::ONE * 2.)
            .with(color(), vec4(0., 1., 0., 1.))
            .with(cast_shadows(), ())
            .spawn();
    }

    let statue = Entity::new()
        .with(
            model_from_url(),
            packages::this::assets::url("hermaneubis.glb"),
        )
        .with(translation(), vec3(0., 0., 0.))
        .with(cast_shadows(), ())
        .with(turntable(), 0.0)
        .spawn();

    App::el(sun, statue, orbitcamera).spawn_interactive();
}

#[element_component]
fn App(hooks: &mut Hooks, sun: EntityId, statue: EntityId, ocam: EntityId) -> Element {
    FlowColumn::el([
        FlowRow::el([
            Text::el("Fog density: "),
            Slider::new_for_entity_component(hooks, sun, fog_density()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog height falloff: "),
            Slider::new_for_entity_component(hooks, sun, fog_height_falloff()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog colour (fog_r): "),
            Slider::new_for_entity_component(hooks, sun, fog_r()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog colour (fog_g): "),
            Slider::new_for_entity_component(hooks, sun, fog_g()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog colour (fog_b): "),
            Slider::new_for_entity_component(hooks, sun, fog_b()).el(),
        ]),
        FlowRow::el([
            Text::el("Sunlight colour (sun_r): "),
            Slider::new_for_entity_component(hooks, sun, sun_r()).el(),
        ]),
        FlowRow::el([
            Text::el("Sunlight colour (sun_g): "),
            Slider::new_for_entity_component(hooks, sun, sun_g()).el(),
        ]),
        FlowRow::el([
            Text::el("Sunlight colour (sun_b): "),
            Slider::new_for_entity_component(hooks, sun, sun_b()).el(),
        ]),
        FlowRow::el([
            Text::el("Ambient colour (amb_r): "),
            Slider::new_for_entity_component(hooks, sun, amb_r()).el(),
        ]),
        FlowRow::el([
            Text::el("Ambient colour (amb_g): "),
            Slider::new_for_entity_component(hooks, sun, amb_g()).el(),
        ]),
        FlowRow::el([
            Text::el("Ambient colour (amb_b): "),
            Slider::new_for_entity_component(hooks, sun, amb_b()).el(),
        ]),
        FlowRow::el([
            Text::el("Sun angle (sun_rotx): "),
            Slider::new_for_entity_component(hooks, sun, sun_rotx()).el(),
        ]),
        FlowRow::el([
            Text::el("Sun angle (sun_roty): "),
            Slider::new_for_entity_component(hooks, sun, sun_roty()).el(),
        ]),
        FlowRow::el([
            Text::el("Sun angle (sun_rotz): "),
            Slider::new_for_entity_component(hooks, sun, sun_rotz()).el(),
        ]),
        FlowRow::el([
            Text::el("Orbit camera turn: "),
            Slider::new_for_entity_component(hooks, ocam, orbit_turn()).el(),
        ]),
        FlowRow::el([
            Text::el("Orbit camera dist: "),
            Slider::new_for_entity_component(hooks, ocam, orbit_zoom()).el(),
        ]),
        FlowRow::el([
            Text::el("Orbit camera pitch: "),
            Slider::new_for_entity_component(hooks, ocam, orbit_pitch()).el(),
        ]),
        FlowRow::el([
            Text::el("Orbit camera target height: "),
            Slider::new_for_entity_component(hooks, ocam, orbit_target_height()).el(),
        ]),
        FlowRow::el([
            Text::el("Model turn: "),
            Slider::new_for_entity_component(hooks, statue, turntable()).el(),
        ]),
    ])
    .with_background(vec4(0., 0., 0., 0.9))
    .with_padding_even(10.)
}
