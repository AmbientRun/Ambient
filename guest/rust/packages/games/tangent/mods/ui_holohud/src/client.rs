use ambient_api::{
    core::{
        app::components::main_scene,
        hierarchy::components::parent,
        rendering::components::color,
        transform::components::{
            local_to_parent, local_to_world, mesh_to_local, mesh_to_world, rotation, translation,
        },
    },
    element::use_entity_component,
    prelude::*,
};
use packages::{
    game_object::components::{health, max_health},
    tangent_schema::{
        character::components as cc, player::components as pc, vehicle::client::components as vcc,
        vehicle::components as vc, vehicle::def as vd,
    },
};

#[main]
pub fn main() {
    Hud.el().spawn_interactive();
}

#[element_component]
fn Hud(hooks: &mut Hooks) -> Element {
    let vehicle_id = use_entity_component(hooks, player::get_local(), pc::vehicle_ref());
    let character_id = use_entity_component(hooks, player::get_local(), pc::character_ref());
    vehicle_id
        .map(VehicleHud::el)
        .or_else(|| character_id.map(CharacterHud::el))
        .unwrap_or_default()
}

#[element_component]
fn VehicleHud(hooks: &mut Hooks, vehicle_id: EntityId) -> Element {
    let health = use_entity_component(hooks, vehicle_id, self::health()).unwrap_or_default();

    let def_ref = use_entity_component(hooks, vehicle_id, vc::def_ref()).unwrap_or_default();

    let name = use_entity_component(hooks, def_ref, vd::components::name())
        .unwrap_or_else(|| "Unknown".to_string());
    let max_health = use_entity_component(hooks, def_ref, max_health()).unwrap_or_default();
    let speed = use_entity_component(hooks, vehicle_id, vcc::speed_kph()).unwrap_or_default();

    let health_color = vec3(0.86, 0.08, 0.24)
        .lerp(vec3(0.54, 0.72, 0.00), health / max_health)
        .extend(1.0);
    let speed_color = Vec4::ONE;

    Group::el([
        Text3D::el(name.to_string(), 3.0)
            .with(color(), speed_color)
            .with(translation(), vec3(0.0, -0.24, 0.0)),
        Text3D::el(format!("{health:.0}"), 4.5)
            .with(color(), health_color)
            .with(translation(), vec3(0.0, -0.12, 0.0)),
        Text3D::el(format!("{speed:.1}"), 3.0)
            .with(color(), speed_color)
            .with(translation(), vec3(0.0, 0.12, 0.0)),
    ])
    .with(local_to_world(), default())
    .with(local_to_parent(), default())
    .with(mesh_to_local(), default())
    .with(mesh_to_world(), default())
    .with(main_scene(), ())
    .with(translation(), vec3(0.0, 2.25, 0.75))
    .with(rotation(), Quat::from_rotation_x(-90.0f32.to_radians()))
    .with(parent(), vehicle_id)
}

#[element_component]
fn CharacterHud(hooks: &mut Hooks, character_id: EntityId) -> Element {
    let health = use_entity_component(hooks, character_id, self::health()).unwrap_or_default();
    let def_ref = use_entity_component(hooks, character_id, cc::def_ref()).unwrap_or_default();
    let max_health = use_entity_component(hooks, def_ref, max_health()).unwrap_or(100.);

    let health_color = vec3(0.86, 0.08, 0.24)
        .lerp(vec3(0.54, 0.72, 0.00), health / max_health)
        .extend(1.0);

    Group::el([Text3D::el(format!("{health:.0}"), 4.0)
        .with(color(), health_color)
        .with(translation(), vec3(0.0, 0.0, 0.0))])
    .with(local_to_world(), default())
    .with(local_to_parent(), default())
    .with(mesh_to_local(), default())
    .with(mesh_to_world(), default())
    .with(main_scene(), ())
    .with(translation(), vec3(-0.25, 0.0, 1.5))
    .with(
        rotation(),
        Quat::from_rotation_z(90f32.to_radians()) * Quat::from_rotation_x(-90.0f32.to_radians()),
    )
    .with(parent(), character_id)
}
