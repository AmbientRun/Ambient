use ambient_api::{
    core::{
        layout::components::{
            align_horizontal, align_vertical, fit_horizontal, fit_vertical, height,
            space_between_items, width,
        },
        text::components::font_size,
    },
    prelude::*,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    let background = |e| FlowRow::el([e]).with_background(vec4(1., 1., 1., 0.02));
    FlowColumn::el([
        FlowRow::el([Text::el("Basic")])
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .with(fit_vertical(), Fit::Children)
            .with(fit_horizontal(), Fit::Children)
            .with_padding_even(10.),
        FlowRow::el([Text::el("Spacing"), Text::el("between"), Text::el("items")])
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .with(fit_vertical(), Fit::Children)
            .with(fit_horizontal(), Fit::Children)
            .with_padding_even(10.)
            .with(space_between_items(), 50.),
        FlowRow::el([Text::el("Break"), Text::el("line")])
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .with(fit_vertical(), Fit::Children)
            .with(fit_horizontal(), Fit::None)
            .with(width(), 50.)
            .with_padding_even(10.),
        FlowRow::el([
            background(Text::el("Align")),
            background(Text::el("Center").with(font_size(), 30.)),
        ])
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with(fit_vertical(), Fit::None)
        .with(fit_horizontal(), Fit::None)
        .with(align_horizontal(), Align::Center)
        .with(align_vertical(), Align::Center)
        .with(width(), 200.)
        .with(height(), 70.)
        .with_padding_even(10.)
        .with(space_between_items(), 5.),
        FlowRow::el([
            background(Text::el("Align")),
            background(Text::el("End").with(font_size(), 30.)),
        ])
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with(fit_vertical(), Fit::None)
        .with(fit_horizontal(), Fit::None)
        .with(align_horizontal(), Align::End)
        .with(align_vertical(), Align::End)
        .with(width(), 200.)
        .with(height(), 70.)
        .with_padding_even(10.)
        .with(space_between_items(), 5.),
    ])
    .with(space_between_items(), 5.)
    .with_padding_even(STREET)
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
