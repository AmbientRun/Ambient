use ambient_element::{element_component, Element, Hooks};
use ambient_guest_bridge::components::{
    transform::{local_to_parent, local_to_world, mesh_to_world, translation},
    ui::{height, width},
};
use glam::vec3;

pub mod text;

#[element_component]
pub fn UIBase(_: &mut Hooks) -> Element {
    Element::new()
        .init(translation(), vec3(0., 0., -0.001))
        .init_default(local_to_world())
        .init_default(local_to_parent())
        .init_default(mesh_to_world())
        .init(width(), 0.)
        .init(height(), 0.)
}
