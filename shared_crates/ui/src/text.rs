//! Defines a text element.

use crate::{UIBase, UIElement};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::core::{
    app::components::{main_scene, name, ui_scene},
    layout::components::{height, width},
    rendering::components::color,
    text::components::{font_family, font_size, text},
    transform::components::{local_to_parent, local_to_world, mesh_to_local, mesh_to_world, scale},
};
use glam::{vec4, Mat4, Vec3};

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

#[element_component]
/// A text element that renders in the main scene in 3D.
pub fn Text3D(
    _hooks: &mut Hooks,
    /// The text to render.
    text: String,
    /// The scale of the text, where 1.0 is about 0.5m tall.
    // TODO: update this to be accurate/precise.
    scale: f32,
) -> Element {
    Element::new()
        .with(local_to_world(), Default::default())
        .with(local_to_parent(), Default::default())
        .with(mesh_to_local(), Default::default())
        .with(mesh_to_world(), Default::default())
        .with(main_scene(), ())
        .with(font_size(), 48.0)
        .with(self::text(), text)
        .with(self::scale(), Vec3::ONE * (scale / 1_000.))
        .init(width(), 1.0)
}
