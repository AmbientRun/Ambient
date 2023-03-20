use glam::{vec4, Vec4};

use crate::UIExt;
use ambient_color::Color;
use ambient_element::Element;
use ambient_guest_bridge::components::{
    layout::{align_vertical_center, space_between_items},
    rect::border_radius,
    rendering::color,
    text::font_size,
};

pub fn primary_color() -> Color {
    Color::hex("DE0B5D").unwrap()
}
pub fn secondary_color() -> Color {
    Color::hex("ffac04").unwrap()
}
pub fn app_background_color() -> Color {
    Color::hex("1B1B1B").unwrap()
}
pub fn error_color() -> Color {
    Color::hex("750631").unwrap()
}
/// A color slightly darker than the app_background
pub fn cutout_color() -> Color {
    Color::hex("151515").unwrap()
}
pub fn tooltip_background_color() -> Color {
    Color::rgba(0., 0., 0., 0.9)
}

/// Default margin/padding
pub const STREET: f32 = 10.;
/// Default rounding of corners
pub const SMALL_ROUNDING: f32 = 3.;

pub trait StylesExt {
    fn section_style(self) -> Self;
    fn header_style(self) -> Self;
    fn small_style(self) -> Self;
    fn error_text_style(self) -> Self;
    fn floating_panel(self) -> Self;
    fn panel(self) -> Self;
    /// A list of items with some space between them
    fn keyboard(self) -> Self;
}
impl StylesExt for Element {
    fn section_style(self) -> Self {
        self.set(font_size(), 16.).set(color(), vec4(0.9, 0.9, 0.9, 1.))
    }
    fn header_style(self) -> Self {
        self.set(font_size(), 25.).set(color(), vec4(0.9, 0.9, 0.9, 1.))
    }
    fn small_style(self) -> Self {
        self.set(font_size(), 10.).set(color(), vec4(0.5, 0.5, 0.5, 1.))
    }
    fn error_text_style(self) -> Self {
        self.set(color(), vec4(1., 0.5, 0.5, 1.))
    }
    #[allow(clippy::clone_on_copy)]
    fn floating_panel(self) -> Self {
        self.with_background(Color::hex("1D1C22").unwrap().set_a(0.9).clone().into())
            .set(border_radius(), Vec4::ONE * 5.)
            .with_padding_even(STREET)
    }
    fn panel(self) -> Self {
        self.with_background(Color::rgba(1., 1., 1., 0.01).into()).set(border_radius(), Vec4::ONE * 5.)
    }
    fn keyboard(self) -> Self {
        self.set(space_between_items(), STREET).with_padding_even(STREET).set_default(align_vertical_center())
    }
}

pub const COLLECTION_ADD_ICON: &str = "\u{f055}";
pub const COLLECTION_DELETE_ICON: &str = "\u{f6bf}";
pub const MOVE_UP_ICON: &str = "\u{f062}";
pub const MOVE_DOWN_ICON: &str = "\u{f063}";
