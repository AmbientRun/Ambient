use ambient_element::{element_component, Element, Hooks};
use ambient_guest_bridge::components::{
    app::{window_logical_size, window_physical_size},
    transform::{local_to_parent, local_to_world, mesh_to_world, translation},
    ui::{height, width},
};
use glam::{vec3, UVec2};

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

/// This only exists so that we can implement From<String> for Text, and then use it in
/// for instance Button
pub struct UIElement(pub Element);
impl From<Element> for UIElement {
    fn from(el: Element) -> Self {
        Self(el)
    }
}

pub fn use_window_physical_resolution(hooks: &mut Hooks) -> UVec2 {
    let (res, set_res) = hooks.use_state(hooks.world.resource(window_physical_size()).clone());
    hooks.use_frame(move |world| {
        let new_res = world.resource(window_physical_size()).clone();
        if new_res != res {
            set_res(new_res);
        }
    });
    res
}
pub fn use_window_logical_resolution(hooks: &mut Hooks) -> UVec2 {
    let (res, set_res) = hooks.use_state(hooks.world.resource(window_logical_size()).clone());
    hooks.use_frame(move |world| {
        let new_res = world.resource(window_logical_size()).clone();
        if new_res != res {
            set_res(new_res);
        }
    });
    res
}
