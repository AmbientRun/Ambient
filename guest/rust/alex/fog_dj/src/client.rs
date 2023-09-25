use ambient_api::{
    core::{
        rendering::components::{
            fog_color, fog_density, fog_height_falloff, light_ambient, light_diffuse,
        },
        transform::components::rotation,
    },
    glam::EulerRot,
    prelude::*,
};
use packages::this::components::{
    amb_b, amb_g, amb_r, fog_b, fog_dj_for, fog_g, fog_r, sun_b, sun_g, sun_r, sun_rotx, sun_roty,
    sun_rotz,
};

#[main]
async fn main() {
    let dj = packages::this::entity();
    let _discardedfuture = entity::wait_for_component(dj, fog_dj_for()).await;

    let sun = entity::get_component(dj, fog_dj_for()).unwrap();
    let sun_rgb = entity::get_component(sun, light_diffuse()).unwrap_or(Vec3::ONE);
    let amb_rgb = entity::get_component(sun, light_ambient()).unwrap_or(Vec3::ONE);
    let sun_rotxyz = entity::get_component(sun, rotation())
        .unwrap_or(Quat::IDENTITY)
        .to_euler(glam::EulerRot::XYZ);
    let fog_rgb = entity::get_component(sun, fog_color()).unwrap_or(Vec3::ONE);
    let starting_fog_density = entity::get_component(sun, fog_density()).unwrap_or(0.);
    let starting_fog_height_falloff =
        entity::get_component(sun, fog_height_falloff()).unwrap_or(0.1);
    entity::add_components(
        dj,
        Entity::new()
            .with(sun_r(), sun_rgb.x)
            .with(sun_g(), sun_rgb.y)
            .with(sun_b(), sun_rgb.z)
            .with(amb_r(), amb_rgb.x)
            .with(amb_g(), amb_rgb.y)
            .with(amb_b(), amb_rgb.z)
            .with(sun_rotx(), sun_rotxyz.0 / 6.28)
            .with(sun_roty(), sun_rotxyz.1 / 6.28)
            .with(sun_rotz(), sun_rotxyz.2 / 6.28)
            .with(fog_r(), fog_rgb.x)
            .with(fog_g(), fog_rgb.y)
            .with(fog_b(), fog_rgb.z)
            .with(fog_density(), starting_fog_density)
            .with(fog_height_falloff(), starting_fog_height_falloff),
    );
    App::el(dj).spawn_interactive();

    query((sun_r(), sun_g(), sun_b())).each_frame(|djs| {
        for (dj, (sr, sg, sb)) in djs {
            entity::add_component(dj, light_diffuse(), vec3(sr, sg, sb));
        }
    });

    query((amb_r(), amb_g(), amb_b())).each_frame(|djs| {
        for (dj, (ar, ag, ab)) in djs {
            entity::add_component(dj, light_ambient(), vec3(ar, ag, ab));
        }
    });

    query((fog_r(), fog_g(), fog_b())).each_frame(|djs| {
        for (dj, (fr, fg, fb)) in djs {
            entity::add_component(dj, fog_color(), vec3(fr, fg, fb));
        }
    });

    query((sun_rotx(), sun_roty(), sun_rotz())).each_frame(|djs| {
        for (dj, (rx, ry, rz)) in djs {
            entity::add_component(
                dj,
                rotation(),
                Quat::from_euler(EulerRot::XYZ, rx * 6.28, ry * 6.28, rz * 6.28),
            );
        }
    });

    query((
        fog_dj_for(),
        light_diffuse(),
        light_ambient(),
        rotation(),
        fog_color(),
        fog_density(),
        fog_height_falloff(),
    ))
    .each_frame(|djs| {
        for (_, (sun, sunlight, sunamb, sunrot, sunfog, sunfogdensity, sunfogheightfalloff)) in djs
        {
            entity::add_component(sun, light_diffuse(), sunlight);
            entity::add_component(sun, light_ambient(), sunamb);
            entity::add_component(sun, rotation(), sunrot);
            entity::add_component(sun, fog_color(), sunfog);
            entity::add_component(sun, fog_density(), sunfogdensity);
            entity::add_component(sun, fog_height_falloff(), sunfogheightfalloff);
        }
    });
}

#[element_component]
fn App(hooks: &mut Hooks, dj: EntityId) -> Element {
    FlowColumn::el([
        FlowRow::el([
            Text::el("Fog density: "),
            Slider::new_for_entity_component(hooks, dj, fog_density()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog height falloff: "),
            Slider::new_for_entity_component(hooks, dj, fog_height_falloff()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog colour (fog_r): "),
            Slider::new_for_entity_component(hooks, dj, fog_r()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog colour (fog_g): "),
            Slider::new_for_entity_component(hooks, dj, fog_g()).el(),
        ]),
        FlowRow::el([
            Text::el("Fog colour (fog_b): "),
            Slider::new_for_entity_component(hooks, dj, fog_b()).el(),
        ]),
        FlowRow::el([
            Text::el("Sunlight colour (sun_r): "),
            Slider::new_for_entity_component(hooks, dj, sun_r()).el(),
        ]),
        FlowRow::el([
            Text::el("Sunlight colour (sun_g): "),
            Slider::new_for_entity_component(hooks, dj, sun_g()).el(),
        ]),
        FlowRow::el([
            Text::el("Sunlight colour (sun_b): "),
            Slider::new_for_entity_component(hooks, dj, sun_b()).el(),
        ]),
        FlowRow::el([
            Text::el("Ambient colour (amb_r): "),
            Slider::new_for_entity_component(hooks, dj, amb_r()).el(),
        ]),
        FlowRow::el([
            Text::el("Ambient colour (amb_g): "),
            Slider::new_for_entity_component(hooks, dj, amb_g()).el(),
        ]),
        FlowRow::el([
            Text::el("Ambient colour (amb_b): "),
            Slider::new_for_entity_component(hooks, dj, amb_b()).el(),
        ]),
        FlowRow::el([
            Text::el("Sun angle (sun_rotx): "),
            Slider::new_for_entity_component(hooks, dj, sun_rotx()).el(),
        ]),
        FlowRow::el([
            Text::el("Sun angle (sun_roty): "),
            Slider::new_for_entity_component(hooks, dj, sun_roty()).el(),
        ]),
        FlowRow::el([
            Text::el("Sun angle (sun_rotz): "),
            Slider::new_for_entity_component(hooks, dj, sun_rotz()).el(),
        ]),
    ])
    .with_background(vec4(0., 0., 0., 0.9))
    .with_padding_even(10.)
}
