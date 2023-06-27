use ambient_api::{
    components::core::layout::{
        docking_bottom, docking_left, fit_horizontal_none, fit_vertical_none, height, width,
    },
    prelude::*,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    let background = |e| {
        FlowRow::el([e])
            .with_background(vec4(1., 1., 1., 0.02))
            .with_default(fit_vertical_none())
            .with_default(fit_horizontal_none())
    };
    Dock::el([
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
        Dock::el([background(Text::el("Fourth padding"))])
            .with_padding_even(10.)
            .with(height(), 70.)
            .with_background(vec4(1., 1., 1., 0.02)),
        background(Text::el("Fill remainder")).with_margin_even(30.),
    ])
    .with_background(vec4(1., 1., 1., 0.02))
    .with_padding_even(STREET)
    .with(width(), 500.)
    .with(height(), 500.)
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
