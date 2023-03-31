use ambient_api::prelude::*;
use ambient_ui_components::prelude::*;

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    let background = |e| FlowRow::el([e]).with_background(vec4(1., 1., 1., 0.02));
    FlowColumn::el([
        FlowRow::el([Text::el("Basic")])
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .with_default(fit_vertical_children())
            .with_default(fit_horizontal_children())
            .with_padding_even(10.),
        FlowRow::el([Text::el("Spacing"), Text::el("between"), Text::el("items")])
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .with_default(fit_vertical_children())
            .with_default(fit_horizontal_children())
            .with_padding_even(10.)
            .with(space_between_items(), 50.),
        FlowRow::el([Text::el("Break"), Text::el("line")])
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .with_default(fit_vertical_children())
            .with_default(fit_horizontal_none())
            .with(width(), 50.)
            .with_padding_even(10.),
        FlowRow::el([
            background(Text::el("Align")),
            background(Text::el("Center").with(font_size(), 30.)),
        ])
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with_default(fit_vertical_none())
        .with_default(fit_horizontal_none())
        .with_default(align_horizontal_center())
        .with_default(align_vertical_center())
        .with(width(), 200.)
        .with(height(), 70.)
        .with_padding_even(10.)
        .with(space_between_items(), 5.),
        FlowRow::el([
            background(Text::el("Align")),
            background(Text::el("End").with(font_size(), 30.)),
        ])
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with_default(fit_vertical_none())
        .with_default(fit_horizontal_none())
        .with_default(align_horizontal_end())
        .with_default(align_vertical_end())
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
