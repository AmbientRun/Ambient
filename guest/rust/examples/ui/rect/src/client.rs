use ambient_api::{
    core::{
        layout::components::{height, space_between_items, width},
        rect::components::{
            background_color, border_color, border_radius, border_thickness, line_from, line_to,
            line_width,
        },
    },
    prelude::*,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    Group::el([
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
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
