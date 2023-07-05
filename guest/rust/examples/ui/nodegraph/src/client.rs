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
    // let (text, set_text) = _hooks.use_state("".to_string());

    // let editor = TextEditor::new(text.clone(), set_text.clone())
    //     .placeholder(Some("type your node..."))
    //     .auto_focus()
    //     .el()
    //     .with(width(), 100.)
    //     .with(height(), 60.)
    //     .with(background_color(), vec4(0.2, 0.6, 0.2, 0.6))
    //     .with(translation(), vec3(0., 40., 0.));

    // .with_default(align_horizontal_center())
    // .with_default(align_vertical_center());

    // let body = Rectangle::el()
    //     .with(width(), 100.)
    //     .with(height(), 60.)
    //     .with(translation(), vec3(0., 30., 0.))
    //     .with(background_color(), vec4(0.2, 0.2, 0.6, 0.6));
    // .children(vec![text]);

    // let dragarea = Rectangle
    //     .el()
    //     .with(width(), 100.)
    //     .with(height(), 30.)
    //     .with(background_color(), vec4(0.6, 0.2, 0.2, 0.6))
    //     .children(vec![body, editor]);
    // FocusRoot::el([dragarea.with_dragarea().el()])

    // Node::el()

    Graph::el()
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
