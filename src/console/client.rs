use ambient_api::{
    components::core::{
        layout::{docking_bottom, docking_fill, fit_horizontal_parent, margin, min_height},
        rendering::color,
    },
    prelude::*,
};

#[main]
pub fn main() {
    App.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    let (toggle, set_toggle) = hooks.use_state(false);
    hooks.use_keyboard_input(move |_, keycode, modifiers, pressed| {
        if modifiers == ModifiersState::empty()
            && keycode == Some(VirtualKeyCode::Grave)
            && !pressed
        {
            set_toggle(!toggle);
        }
    });

    if toggle {
        Console::el()
    } else {
        Element::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum ConsoleLineColor {
    Normal,
    User,
}
impl From<ConsoleLineColor> for Vec4 {
    fn from(value: ConsoleLineColor) -> Self {
        match value {
            ConsoleLineColor::Normal => vec4(0.8, 0.8, 0.8, 1.0),
            ConsoleLineColor::User => vec4(0.0, 0.8, 0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
struct ConsoleLine {
    text: String,
    color: ConsoleLineColor,
}

#[element_component]
pub fn Console(hooks: &mut Hooks) -> Element {
    hooks.use_spawn(|_| {
        messages::RequestInput {}.send_local_broadcast(false);
        |_| {
            messages::ReleaseInput {}.send_local_broadcast(false);
        }
    });
    let (command, set_command) = hooks.use_state_with(|_| String::new());
    let (messages, set_messages) = hooks.use_state_with(|_| {
        (0..10)
            .map(|i| ConsoleLine {
                text: format!("Line {i}"),
                color: ConsoleLineColor::Normal,
            })
            .collect::<Vec<_>>()
    });

    FocusRoot::el([WindowSized::el([with_rect(Dock::el([
        // text entry
        TextEditor::new(command, set_command.clone())
            .auto_focus()
            .placeholder(Some("Enter command..."))
            .on_submit({
                let messages = messages.clone();
                move |text| {
                    let mut new_messages = messages.clone();
                    new_messages.push(ConsoleLine {
                        text: format!("> {}", text),
                        color: ConsoleLineColor::User,
                    });
                    set_messages(new_messages);
                    set_command(String::new());
                }
            })
            .el()
            .with_background(vec4(0.0, 0.0, 0.0, 0.5))
            .with_padding_even(4.0)
            .with_default(fit_horizontal_parent())
            .with_default(docking_bottom())
            .with(min_height(), 22.0)
            .with(margin(), vec4(STREET, STREET, 0.0, STREET)),
        // log
        ScrollArea::el(
            ScrollAreaSizing::FitParentWidth,
            FlowColumn::el(
                messages
                    .into_iter()
                    .map(|m| Text::el(m.text).with(color(), m.color.into())),
            ),
        )
        .with_background(vec4(0.0, 0.0, 0.0, 0.5))
        .with_default(docking_fill())
        .with_margin_even(STREET),
    ]))
    .with_background(vec4(0.0, 0.0, 0.0, 0.5))])
    .with_padding_even(20.)])
}
