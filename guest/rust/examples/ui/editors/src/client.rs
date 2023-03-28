use ambient_api::prelude::*;
use ambient_cb::cb;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::layout::{min_width, space_between_items, width};
use ambient_ui_components::{
    default_theme::STREET,
    editor::{Editor, F32Input, ListEditor, MinimalListEditor, TextEditor},
    layout::{FlowColumn, FlowRow},
    select::DropdownSelect,
    text::Text,
    FocusRoot, UIExt,
};
use indexmap::IndexMap;

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (text, set_text) = hooks.use_state("Enter some text".to_string());
    let (float, set_float) = hooks.use_state(0.0);
    let (vector3, set_vector3) = hooks.use_state(Vec3::ZERO);
    let (index_map, set_index_map) = hooks.use_state(
        vec![("First".to_string(), "Second".to_string())]
            .into_iter()
            .collect::<IndexMap<String, String>>(),
    );
    let (list, set_list) = hooks.use_state(vec!["First".to_string(), "Second".to_string()]);
    let (minimal_list, set_minimal_list) =
        hooks.use_state(vec!["First".to_string(), "Second".to_string()]);
    let row = |name, editor| FlowRow(vec![Text::el(name).with(min_width(), 110.), editor]).el();
    FocusRoot(vec![FlowColumn(vec![
        row("TextEditor", TextEditor::new(text, set_text).el()),
        row(
            "F32Input",
            F32Input {
                value: float,
                on_change: set_float,
            }
            .el(),
        ),
        row(
            "DropDownSelect",
            DropdownSelect {
                content: Text::el("Select"),
                on_select: cb(|_| {}),
                items: vec![Text::el("First"), Text::el("Second")],
                inline: false,
            }
            .el(),
        ),
        row(
            "Vec3",
            Vec3::editor(vector3, set_vector3, Default::default()),
        ),
        row(
            "IndexMap",
            IndexMap::editor(index_map, set_index_map, Default::default()),
        ),
        row(
            "ListEditor",
            ListEditor {
                value: list,
                on_change: Some(set_list),
            }
            .el(),
        ),
        row(
            "MinimalListEditor",
            MinimalListEditor {
                value: minimal_list,
                on_change: Some(set_minimal_list),
                item_opts: Default::default(),
                add_presets: None,
                add_title: "Add".to_string(),
            }
            .el(),
        ),
    ])
    .el()
    .with(width(), 200.)
    .with(space_between_items(), STREET)
    .with_padding_even(STREET)])
    .el()
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
