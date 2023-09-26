pub mod package_load;
pub mod package_manager;
pub mod package_view;
pub mod util;

use ambient_api::{
    prelude::{cb, vec4, WindowStyle},
    ui::UIExt,
};
use ambient_color::Color;
use ambient_design_tokens::LIGHT::{
    SEMANTIC_MAIN_SURFACE_PRIMARY, SEMANTIC_MAIN_SURFACE_SECONDARY,
};
pub use util::*;

pub fn window_style() -> WindowStyle {
    WindowStyle {
        body: cb(|e| {
            e.with_background(Color::hex(SEMANTIC_MAIN_SURFACE_SECONDARY).unwrap().into())
        }),
        title_bar: cb(|e| {
            e.with_background(Color::hex(SEMANTIC_MAIN_SURFACE_PRIMARY).unwrap().into())
        }),
        title_text: cb(|e| e),
    }
}
