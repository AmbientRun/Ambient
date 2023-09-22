use ambient_api::{
    core::{
        app::components::main_scene,
        hierarchy::components::parent,
        rendering::components::color,
        text::components::{font_size, text},
        transform::components::{
            local_to_parent, local_to_world, mesh_to_local, mesh_to_world, rotation, scale,
            translation,
        },
    },
    element::use_entity_component,
    prelude::*,
};
use packages::tangent_schema::{
    player::components as pc, vehicle::client::components as vcc, vehicle::components as vc,
    vehicle::data as vdc,
};

#[main]
pub fn main() {
    Hud.el().spawn_interactive();
}

#[element_component]
fn Hud(hooks: &mut Hooks) -> Element {
    let vehicle_id = use_entity_component(hooks, player::get_local(), pc::vehicle_ref());
    vehicle_id.map(VehicleHud::el).unwrap_or_default()
}

#[element_component]
fn VehicleHud(hooks: &mut Hooks, vehicle_id: EntityId) -> Element {
    let health = use_entity_component(hooks, vehicle_id, vc::health()).unwrap_or_default();

    let max_health =
        use_entity_component(hooks, vehicle_id, vdc::general::components::max_health())
            .unwrap_or_default();

    let speed = use_entity_component(hooks, vehicle_id, vcc::speed_kph()).unwrap_or_default();

    let health_color = vec3(0.86, 0.08, 0.24)
        .lerp(vec3(0.54, 0.72, 0.00), health / max_health)
        .extend(1.0);
    let speed_color = Vec4::ONE;

    Group::el([
        Text3D::el(format!("{health:.0}"), health_color, 1.5)
            .with(translation(), vec3(0.0, -0.04, 0.0)),
        Text3D::el(format!("{speed:.1}"), speed_color, 1.0)
            .with(translation(), vec3(0.0, 0.04, 0.0)),
    ])
    .with(local_to_world(), default())
    .with(local_to_parent(), default())
    .with(mesh_to_local(), default())
    .with(mesh_to_world(), default())
    .with(main_scene(), ())
    .with(translation(), vec3(0.0, 0.75, 0.25))
    .with(rotation(), Quat::from_rotation_x(-90.0f32.to_radians()))
    .with(parent(), vehicle_id)
}

#[element_component]
fn Text3D(_hooks: &mut Hooks, text: String, color: Vec4, scale: f32) -> Element {
    Element::new()
        .with(local_to_world(), default())
        .with(local_to_parent(), default())
        .with(mesh_to_local(), default())
        .with(mesh_to_world(), default())
        .with(main_scene(), ())
        .with(self::text(), text)
        .with(self::color(), color)
        .with(self::scale(), Vec3::ONE * (scale / 1_000.))
        .with(font_size(), 48.0)
}
