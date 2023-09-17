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
    vehicle_data as vdc,
};

#[main]
pub fn main() {
    Hud.el().spawn_interactive();
}

#[element_component]
fn Hud(hooks: &mut Hooks) -> Element {
    let vehicle_id = use_entity_component(hooks, player::get_local(), pc::vehicle_ref()).0;
    vehicle_id.map(VehicleHud::el).unwrap_or_default()
}

#[element_component]
fn VehicleHud(_hooks: &mut Hooks, vehicle_id: EntityId) -> Element {
    Group::el([
        VehicleHudSide::el(vehicle_id),
        VehicleHudBack::el(vehicle_id),
    ])
}

#[element_component]
fn VehicleHudSide(hooks: &mut Hooks, vehicle_id: EntityId) -> Element {
    let health = use_entity_component(hooks, vehicle_id, vc::health())
        .0
        .unwrap_or_default();
    let max_health =
        use_entity_component(hooks, vehicle_id, vdc::general::components::max_health())
            .0
            .unwrap_or_default();

    Group::el([Text3D::el(
        format!("{health:.0} HP"),
        vec3(0.86, 0.08, 0.24)
            .lerp(vec3(0.54, 0.72, 0.00), health / max_health)
            .extend(1.0),
        3.,
    )
    .with(translation(), vec3(0.0, 0.0, 0.0))])
    .with(local_to_world(), default())
    .with(local_to_parent(), default())
    .with(mesh_to_local(), default())
    .with(mesh_to_world(), default())
    .with(main_scene(), ())
    .with(translation(), vec3(0.35, 0.0, 0.3))
    .with(
        rotation(),
        Quat::from_rotation_z(15.0f32.to_radians()) * Quat::from_rotation_x(-65.0f32.to_radians()),
    )
    .with(parent(), vehicle_id)
}

#[element_component]
fn VehicleHudBack(hooks: &mut Hooks, vehicle_id: EntityId) -> Element {
    let kph = use_entity_component(hooks, vehicle_id, vcc::speed_kph())
        .0
        .unwrap_or_default();

    Group::el([Text3D::el(format!("{kph:.1}"), Vec4::ONE, 1.)])
        .with(local_to_world(), default())
        .with(local_to_parent(), default())
        .with(mesh_to_local(), default())
        .with(mesh_to_world(), default())
        .with(main_scene(), ())
        .with(translation(), vec3(0.0, 0.75, 0.225))
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
