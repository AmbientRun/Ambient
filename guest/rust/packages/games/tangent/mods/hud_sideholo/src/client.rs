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
use packages::tangent_schema::{player::components as pc, vehicle::client::components as vcc};

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
fn VehicleHud(hooks: &mut Hooks, vehicle_id: EntityId) -> Element {
    let kph = use_entity_component(hooks, vehicle_id, vcc::speed_kph())
        .0
        .unwrap_or_default();

    Element::new()
        .with(local_to_world(), default())
        .with(translation(), vec3(0.35, 0., 0.3))
        .with(
            rotation(),
            Quat::from_rotation_z(25.0f32.to_radians())
                * Quat::from_rotation_x(-65.0f32.to_radians()),
        )
        .with(scale(), Vec3::ONE * 0.005)
        .with(local_to_parent(), Default::default())
        .with(mesh_to_local(), Default::default())
        .with(mesh_to_world(), Default::default())
        .with(main_scene(), ())
        .with(text(), format!("{:.1}", kph))
        .with(color(), vec4(1., 1., 1., 1.))
        .with(font_size(), 48.0)
        .with(parent(), vehicle_id)
}
