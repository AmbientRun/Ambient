use ambient_api::prelude::*;
use ambient_element::{element_component, Element, ElementComponentExt, Group, Hooks};
use ambient_guest_bridge::components::{
    layout::{height, space_between_items, width},
    rect::{
        background_color, border_color, border_radius, border_thickness, line_from, line_to,
        line_width,
    },
};
use ambient_ui_components::{default_theme::STREET, layout::FlowColumn, Line, Rectangle, UIExt};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    Group(vec![
        FlowColumn::el([
            Rectangle.el(),
            Rectangle
                .el()
                .with(width(), 150.)
                .with(height(), 50.)
                .with(background_color(), vec4(1., 0., 0., 1.))
                .with(border_color(), vec4(0., 1., 0., 1.))
                .with(border_thickness(), 10.)
                .with(border_radius(), vec4(20., 10., 5., 0.)),
        ])
        .with(space_between_items(), 10.)
        .with_padding_even(STREET),
        Line.el()
            .with(line_from(), vec3(200., 200., 0.))
            .with(line_to(), vec3(300., 200., 0.))
            .with(line_width(), 1.)
            .with(background_color(), vec4(1., 0., 0., 1.)),
        Line.el()
            .with(line_from(), vec3(200., 200., 0.))
            .with(line_to(), vec3(200., 300., 0.))
            .with(line_width(), 1.)
            .with(background_color(), vec4(0., 1., 0., 1.)),
        Line.el()
            .with(line_from(), vec3(200., 200., 0.))
            .with(line_to(), vec3(500., 300., 0.))
            .with(line_width(), 10.)
            .with(background_color(), vec4(0., 0., 1., 1.)),
    ])
    .el()
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
