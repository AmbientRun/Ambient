use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::{
    app::{ui_scene, window_logical_size, window_physical_size},
    transform::{local_to_parent, local_to_world, mesh_to_local, mesh_to_world, scale, translation},
    ui::{
        background_color, gpu_ui_size, height, margin_bottom, margin_left, margin_right, margin_top, mesh_to_local_from_size,
        padding_bottom, padding_left, padding_right, padding_top, rect, width,
    },
};
use clickarea::ClickArea;
use glam::{vec3, Mat4, UVec2, Vec3, Vec4};

pub mod clickarea;
pub mod default_theme;
pub mod dropdown;
pub mod layout;
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

// We need `clone` as resource is a ref on host and a copy on guest
#[allow(clippy::clone_on_copy)]
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
// We need `clone` as resource is a ref on host and a copy on guest
#[allow(clippy::clone_on_copy)]
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

/// A simple UI rect. Use components such as `width`, `height`, `background_color`, `border_color`, `border_radius` and `border_thickness`
/// to control its appearance
#[element_component]
pub fn Rectangle(_hooks: &mut Hooks) -> Element {
    with_rect(UIBase.el()).set(width(), 100.).set(height(), 100.).set(background_color(), Vec4::ONE)
}

pub fn with_rect(element: Element) -> Element {
    element
        .init(rect(), ())
        .init(gpu_ui_size(), Vec4::ZERO)
        .init(mesh_to_local(), Mat4::IDENTITY)
        .init(scale(), Vec3::ONE)
        .init(mesh_to_local_from_size(), ())
        .init(ui_scene(), ())
}

pub trait UIExt {
    fn with_clickarea(self) -> ClickArea;
    fn with_background(self, color: Vec4) -> Self;
    fn with_padding_even(self, padding: f32) -> Self;
    fn with_margin_even(self, margin: f32) -> Self;
}
impl UIExt for Element {
    fn with_clickarea(self) -> ClickArea {
        ClickArea::new(self)
    }
    fn with_background(self, background: Vec4) -> Self {
        with_rect(self).set(background_color(), background)
    }
    fn with_padding_even(self, padding: f32) -> Self {
        self.set(padding_left(), padding).set(padding_right(), padding).set(padding_top(), padding).set(padding_bottom(), padding)
    }
    fn with_margin_even(self, margin: f32) -> Self {
        self.set(margin_left(), margin).set(margin_right(), margin).set(margin_top(), margin).set(margin_bottom(), margin)
    }
}
