use ambient_api::{
    core::{
        rendering::components::color,
        text::{
            components::{font_size, font_style},
            types::FontStyle,
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

    WindowSized::el([Dock::el([FlowRow::el([
        Text::el(format!("{health:.0}"))
            .with(color(), health_color)
            .with(font_size(), 36.0),
        FlowColumn::el([
            Text::el(name.to_string())
                .with(color(), Vec4::ONE)
                .with(font_size(), 18.0)
                .with(font_style(), FontStyle::Italic),
            Text::el(format!("{speed:.1}"))
                .with(color(), speed_color)
                .with(font_size(), 18.0),
        ]),
    ])
    .with(space_between_items(), 8.)
    .with(docking(), Docking::Bottom)
    .with_margin_even(10.)])
    .with_padding_even(STREET)])
}

#[element_component]
fn CharacterHud(hooks: &mut Hooks, character_id: EntityId) -> Element {
    let health = use_entity_component(hooks, character_id, self::health()).unwrap_or_default();
    let def_ref = use_entity_component(hooks, character_id, cc::def_ref()).unwrap_or_default();
    let max_health = use_entity_component(hooks, def_ref, max_health()).unwrap_or(100.);

    let health_color = vec3(0.86, 0.08, 0.24)
        .lerp(vec3(0.54, 0.72, 0.00), health / max_health)
        .extend(1.0);

    WindowSized::el([Dock::el([FlowRow::el([Text::el(format!("{health:.0}"))
        .with(color(), health_color)
        .with(font_size(), 36.0)])
    .with(docking(), Docking::Bottom)
    .with_margin_even(10.)])
    .with_padding_even(STREET)])
}
