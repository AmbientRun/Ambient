use ambient_api::{
    components::core::{
        layout::{
            align_horizontal_center, align_vertical_center, height, space_between_items, width,
        },
        rect::{
            background_color, border_color, border_radius, border_thickness, line_from, line_to,
            line_width,
        },
        transform::translation,
    },
    prelude::*,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    Graph::el()
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
