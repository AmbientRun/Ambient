use ambient_api::prelude::*;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::{
    layout::{
        align_horizontal_center, align_horizontal_end, align_vertical_center, align_vertical_end,
        fit_horizontal_children, fit_horizontal_none, fit_vertical_children, fit_vertical_none,
        height, space_between_items, width,
    },
    text::font_size,
};
use ambient_ui_components::{
    default_theme::STREET,
    layout::{FlowColumn, FlowRow},
    text::Text,
    UIExt,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    let background = |e| {
        FlowRow(vec![e])
            .el()
            .with_background(vec4(1., 1., 1., 0.02))
    };
    FlowColumn(vec![
        FlowRow(vec![Text::el("Basic")])
            .el()
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .with_default(fit_vertical_children())
            .with_default(fit_horizontal_children())
            .with_padding_even(10.),
        FlowRow(vec![
            Text::el("Spacing"),
            Text::el("between"),
            Text::el("items"),
        ])
        .el()
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with_default(fit_vertical_children())
        .with_default(fit_horizontal_children())
        .with_padding_even(10.)
        .with(space_between_items(), 50.),
        FlowRow(vec![Text::el("Break"), Text::el("line")])
            .el()
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            .with_default(fit_vertical_children())
            .with_default(fit_horizontal_none())
            .with(width(), 50.)
            .with_padding_even(10.),
        FlowRow(vec![
            background(Text::el("Align")),
            background(Text::el("Center").with(font_size(), 30.)),
        ])
        .el()
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with_default(fit_vertical_none())
        .with_default(fit_horizontal_none())
        .with_default(align_horizontal_center())
        .with_default(align_vertical_center())
        .with(width(), 200.)
        .with(height(), 70.)
        .with_padding_even(10.)
        .with(space_between_items(), 5.),
        FlowRow(vec![
            background(Text::el("Align")),
            background(Text::el("End").with(font_size(), 30.)),
        ])
        .el()
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
    .el()
    .with(space_between_items(), 5.)
    .with_padding_even(STREET)
}

#[main]
pub async fn main() -> EventResult {
    App.el().spawn_interactive();

    EventOk
}
