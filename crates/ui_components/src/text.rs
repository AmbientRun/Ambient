use crate::{UIBase, UIElement};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        app::{name, ui_scene},
        rendering::color,
        ui::{font_size, text},
    },
    components::{
        transform::mesh_to_local,
        ui::{height, width},
    },
};
use glam::{vec4, Mat4};

/// A text element. Use the `text`, `font_size`, `font` and `color` components to set its state
#[element_component(without_el)]
pub fn Text(hooks: &mut Hooks) -> Element {
    UIBase
        .el()
        .init(width(), 1.)
        .init(height(), 1.)
        .init(mesh_to_local(), Mat4::IDENTITY)
        .init(color(), vec4(0.6, 0.6, 0.6, 1.))
        .init(name(), "Text".to_string())
        .init(ui_scene(), ())
        // .init_default(font_family())
        // .init_default(font_style())
        .init(font_size(), 12.)
        .init(text(), "".to_string())
}
impl Text {
    pub fn el(value: impl Into<String>) -> Element {
        Text.el().set(text(), value.into())
    }
}
impl From<&str> for UIElement {
    fn from(value: &str) -> Self {
        UIElement(Text.el().set(text(), value.to_string()))
    }
}
impl From<String> for UIElement {
    fn from(value: String) -> Self {
        UIElement(Text.el().set(text(), value))
    }
}
impl From<&String> for UIElement {
    fn from(value: &String) -> Self {
        UIElement(Text.el().set(text(), value.to_string()))
    }
}
