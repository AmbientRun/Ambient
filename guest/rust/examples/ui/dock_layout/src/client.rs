use ambient_api::prelude::*;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::layout::{
    docking_bottom, docking_left, fit_horizontal_none, fit_vertical_none, height, width,
};
use ambient_ui_components::{
    default_theme::STREET,
    layout::{Dock, FlowRow},
    text::Text,
    UIExt,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    let background = |e| {
        FlowRow(vec![e])
            .el()
            .with_background(vec4(1., 1., 1., 0.02))
            .with_default(fit_vertical_none())
            .with_default(fit_horizontal_none())
    };
    Dock(vec![
        background(Text::el("First"))
            .with(height(), 30.)
            .with_margin_even(10.),
        background(Text::el("Second bottom"))
            .with_default(docking_bottom())
            .with(height(), 50.)
            .with_margin_even(10.),
        background(Text::el("Third left"))
            .with_default(docking_left())
            .with(width(), 70.),
        Dock(vec![background(Text::el("Fourth padding"))])
            .el()
            .with_padding_even(10.)
            .with(height(), 70.)
            .with_background(vec4(1., 1., 1., 0.02)),
        background(Text::el("Fill remainder")).with_margin_even(30.),
    ])
    .el()
    .with_background(vec4(1., 1., 1., 0.02))
    .with_padding_even(STREET)
    .with(width(), 500.)
    .with(height(), 500.)
}

#[main]
pub fn main() -> ResultEmpty {
    App.el().spawn_interactive();

    OkEmpty
}
