use ambient_api::prelude::*;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::{
    layout::space_between_items, rendering::color, text::font_size,
};
use ambient_ui_components::{
    default_theme::{StylesExt, STREET},
    layout::{FlowColumn, Separator},
    text::Text,
    UIExt,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    FlowColumn::el([
        Text::el("Header").header_style(),
        Text::el("Section").section_style(),
        Text::el("Default text \u{f1e2} \u{fb8f}"),
        Text::el("Small").small_style(),
        Separator { vertical: false }.el(),
        Text::el("Custom size").with(font_size(), 40.),
        Text::el("Custom color").with(color(), vec4(1., 0., 0., 1.)),
        Text::el("Multi\n\nLine"),
    ])
    .with_padding_even(STREET)
    .with(space_between_items(), 10.)
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
