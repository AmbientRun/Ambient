use ambient_api::prelude::*;
use ambient_ui_components::prelude::*;
use components::todo_item;

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    FocusRoot::el([FlowColumn::el([NewTodoItem.el(), TodoItems.el()])
        .with_padding_even(STREET)
        .with(space_between_items(), 10.)])
}

#[element_component]
fn NewTodoItem(hooks: &mut Hooks) -> Element {
    let (text, set_text) = hooks.use_state("".to_string());
    let text_is_empty = text.is_empty();

    FlowColumn::el([
        TextEditor::new(text.clone(), set_text.clone())
            .placeholder(Some("Enter todo name here"))
            .auto_focus()
            .el(),
        Button::new("Create", move |_| {
            messages::NewItem::new(text.clone()).send_server_reliable();
            set_text(String::new());
        })
        .disabled(text_is_empty)
        .el(),
    ])
    .with(space_between_items(), 10.)
}

#[element_component]
fn TodoItems(hooks: &mut Hooks) -> Element {
    let items = hooks.use_query(todo_item());
    FlowColumn::el(items.into_iter().map(|(id, description)| {
        FlowRow::el([
            Button::new(COLLECTION_DELETE_ICON, move |_| {
                messages::DeleteItem::new(id).send_server_reliable()
            })
            .style(ButtonStyle::Flat)
            .el(),
            Text::el(description),
        ])
    }))
    .with(space_between_items(), 10.)
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
