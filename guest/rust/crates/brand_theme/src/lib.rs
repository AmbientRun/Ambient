use ambient_api::core::rect::components::border_radius;
use ambient_api::core::transform::components::translation;
use ambient_api::prelude::*;
use ambient_api::{
    core::{
        rendering::components::color,
        text::components::{font_family, font_size},
    },
    prelude::{cb, element_component, Cb, Element, Hooks, WindowStyle},
    ui::{Rectangle, UIExt},
};
use ambient_color::Color;
pub use ambient_design_tokens as design_tokens;
use ambient_design_tokens::LIGHT::{
    SEMANTIC_MAININVERTED_SURFACE_SECONDARY, SEMANTIC_MAIN_ELEMENTS_PRIMARY,
    SEMANTIC_MAIN_SURFACE_PRIMARY, SEMANTIC_MAIN_SURFACE_SECONDARY,
};
pub fn window_style() -> WindowStyle {
    WindowStyle {
        body: cb(|e| e.hex_background(SEMANTIC_MAIN_SURFACE_SECONDARY)),
        title_bar: cb(|e| e.hex_background(SEMANTIC_MAIN_SURFACE_PRIMARY)),
        title_text: cb(|e| {
            e.mono_xs_500upp()
                .hex_text_color(SEMANTIC_MAIN_ELEMENTS_PRIMARY)
        }),
    }
}

pub trait AmbientInternalStyle {
    fn hex_background(self, hex: &str) -> Self;
    fn hex_text_color(self, hex: &str) -> Self;
    fn mono(self) -> Self;
    fn font_body_medium(self) -> Self;
    fn mono_xs_500upp(self) -> Self;
    fn mono_s_500upp(self) -> Self;
    fn body_s_500(self) -> Self;
}
impl AmbientInternalStyle for Element {
    fn hex_background(self, hex: &str) -> Self {
        self.with_background(Color::hex(hex).unwrap().into())
    }
    fn hex_text_color(self, hex: &str) -> Self {
        self.with(color(), Color::hex(hex).unwrap().into())
    }
    fn mono(self) -> Self {
        self.with(font_family(), "https://internal-content.fra1.cdn.digitaloceanspaces.com/fonts/DiatypeMono/ABCDiatypeMono-Medium.otf".to_string())
    }
    fn font_body_medium(self) -> Self {
        self.with(font_family(), "https://internal-content.fra1.cdn.digitaloceanspaces.com/fonts/ABCDiatypeVariable/Diatype/ABCDiatype-Medium.otf".to_string())
    }
    fn mono_xs_500upp(self) -> Self {
        self.mono().with(font_size(), 12.8)
    }
    fn mono_s_500upp(self) -> Self {
        self.mono().with(font_size(), 16.)
    }
    fn body_s_500(self) -> Self {
        self.font_body_medium().with(font_size(), 16.)
    }
}

pub const SEMANTIC_MAIN_ELEMENTS_TERTIARY: &str = "#595959";

#[element_component]
pub fn Toggle(
    _hooks: &mut Hooks,
    value: bool,
    on_change: Cb<dyn Fn(bool) + Sync + Send>,
) -> Element {
    let outer_width = 54.;
    let outer_height = 32.;
    let thumb = 24.;
    let thumb_margin = 2.;
    let right = outer_width - thumb - thumb_margin;
    let left = thumb_margin;
    Rectangle::el()
        .hex_background(SEMANTIC_MAIN_SURFACE_PRIMARY)
        .with(width(), outer_width)
        .with(height(), outer_height)
        .with(border_radius(), Vec4::ONE * outer_height / 2.)
        .children(vec![Rectangle::el()
            .hex_background(if value {
                SEMANTIC_MAIN_SURFACE_SECONDARY
            } else {
                SEMANTIC_MAININVERTED_SURFACE_SECONDARY
            })
            .with(width(), thumb)
            .with(height(), thumb)
            .with(border_radius(), Vec4::ONE * thumb / 2.)
            .with(
                translation(),
                vec3(
                    if value { left } else { right },
                    (outer_height - thumb) / 2.,
                    -0.0001,
                ),
            )])
        .with_clickarea()
        .on_mouse_down(move |_, _, _| {
            on_change(!value);
        })
        .el()
}
