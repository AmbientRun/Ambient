//! Defines the default theme for the UI.
use glam::{vec4, Vec4};

use crate::UIExt;
use ambient_color::Color;
use ambient_element::Element;
use ambient_guest_bridge::core::{
    layout::components::space_between_items, rect::components::border_radius,
    rendering::components::color, text::components::font_size,
};

/// The primary color.
pub fn primary_color() -> Color {
    Color::hex("DE0B5D").unwrap()
}
/// The secondary color.
pub fn secondary_color() -> Color {
    Color::hex("ffac04").unwrap()
}
/// The color used for the background of the app.
pub fn app_background_color() -> Color {
    Color::hex("1B1B1B").unwrap()
}
/// The error color.
pub fn error_color() -> Color {
    Color::hex("750631").unwrap()
}
/// A color slightly darker than [app_background_color].
pub fn cutout_color() -> Color {
    Color::hex("151515").unwrap()
}
/// The color used for tooltip backgrounds.
pub fn tooltip_background_color() -> Color {
    Color::rgba(0., 0., 0., 0.9)
}

/// Default margin/padding.
pub const STREET: f32 = 10.;
/// Default rounding of corners.
pub const SMALL_ROUNDING: f32 = 3.;

/// A trait that adds some default styles to elements.
pub trait StylesExt {
    /// Apply the default style for a section header.
    fn section_style(self) -> Self;
    /// Apply the default style for a header.
    fn header_style(self) -> Self;
    /// Apply the default style for a small text.
    fn small_style(self) -> Self;
    /// Apply the default style for an error text.
    fn error_text_style(self) -> Self;
    /// Apply the default style for a floating panel.
    fn floating_panel(self) -> Self;
    /// Apply the default style for a panel.
    fn panel(self) -> Self;
    /// A list of items with some space between them.
    fn keyboard(self) -> Self;
}
impl StylesExt for Element {
    fn section_style(self) -> Self {
        self.with(font_size(), 16.)
            .with(color(), vec4(0.9, 0.9, 0.9, 1.))
    }
    fn header_style(self) -> Self {
        self.with(font_size(), 25.)
            .with(color(), vec4(0.9, 0.9, 0.9, 1.))
    }
    fn small_style(self) -> Self {
        self.with(font_size(), 10.)
            .with(color(), vec4(0.5, 0.5, 0.5, 1.))
    }
    fn error_text_style(self) -> Self {
        self.with(color(), vec4(1., 0.5, 0.5, 1.))
    }
    #[allow(clippy::clone_on_copy)]
    fn floating_panel(self) -> Self {
        self.with_background(Color::hex("1D1C22").unwrap().set_a(0.9).clone().into())
            .with(border_radius(), Vec4::ONE * 5.)
            .with_padding_even(STREET)
    }
    fn panel(self) -> Self {
        self.with_background(Color::rgba(1., 1., 1., 0.01).into())
            .with(border_radius(), Vec4::ONE * 5.)
    }
    fn keyboard(self) -> Self {
        self.with(space_between_items(), STREET)
            .with_padding_even(STREET)
        //LEGACY_MISSING_ENUM_SUPPORT: .with_default(align_vertical_center())
    }
}

/// Character code for the "add" icon.
pub const COLLECTION_ADD_ICON: &str = "\u{f055}";
/// Character code for the "delete" icon.
pub const COLLECTION_DELETE_ICON: &str = "\u{f6bf}";
/// Character code for the "move up" icon.
pub const MOVE_UP_ICON: &str = "\u{f062}";
/// Character code for the "move down" icon.
pub const MOVE_DOWN_ICON: &str = "\u{f063}";
/// Character code for the "right chevron" icon.
pub const CHEVRON_RIGHT: &str = "\u{f054}";
/// Character code for the "left chevron" icon.
pub const CHEVRON_LEFT: &str = "\u{f053}";
/// Character code for the "down chevron" icon.
pub const CHEVRON_DOWN: &str = "\u{f078}";
/// Character code for the "up chevron" icon.
pub const CHEVRON_UP: &str = "\u{f077}";
