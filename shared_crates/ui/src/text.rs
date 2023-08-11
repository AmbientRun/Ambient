//! Defines a text element.

use crate::{UIBase, UIElement};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::core::{
    app::components::{name, ui_scene},
    layout::components::{height, width},
    rendering::components::color,
    text::components::{font_family, font_size, text},
    transform::components::mesh_to_local,
};
use glam::{vec4, Mat4};

/// A text element. Use the [text], [font_size], [font_family] and [color] components to set its state.
#[element_component(without_el)]
pub fn Text(_hooks: &mut Hooks) -> Element {
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
    /// Creates a new text element with the given text.
    pub fn el(value: impl Into<String>) -> Element {
        Text.el().with(text(), value.into())
    }
}
impl From<&str> for UIElement {
    fn from(value: &str) -> Self {
        UIElement(Text.el().with(text(), value.to_string()))
    }
}
impl From<String> for UIElement {
    fn from(value: String) -> Self {
        UIElement(Text.el().with(text(), value))
    }
}
impl From<&String> for UIElement {
    fn from(value: &String) -> Self {
        UIElement(Text.el().with(text(), value.to_string()))
    }
}

#[element_component]
/// A FontAwesome icon.
pub fn FontAwesomeIcon(
    _hooks: &mut Hooks,
    /// The icon codepoint.
    icon: u32,
    /// Whether the icon should be solid or not.
    solid: bool,
) -> Element {
    Text::el(char::from_u32(icon).unwrap().to_string()).with(
        font_family(),
        if solid {
            "FontAwesomeSolid"
        } else {
            "FontAwesome"
        }
        .to_string(),
    )
}
