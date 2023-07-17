use std::sync::{Arc, Mutex};

use ambient_api::{
    components::core::{
        layout::{docking_bottom, docking_fill, fit_horizontal_parent, margin, min_height},
        rendering::color,
    },
    prelude::*,
};
use shared::*;

mod shared;

#[main]
pub fn main() {
    let console = Console::new();

    App {
        console: Arc::new(Mutex::new(console)),
    }
    .el()
    .spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks, console: Arc<Mutex<Console>>) -> Element {
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
        ConsoleView::el(console)
    } else {
        Element::new()
    }
}

#[element_component]
pub fn ConsoleView(hooks: &mut Hooks, console: Arc<Mutex<Console>>) -> Element {
    hooks.use_spawn(|_| {
        messages::RequestInput {}.send_local_broadcast(false);
        |_| {
            messages::ReleaseInput {}.send_local_broadcast(false);
        }
    });

    let render_signal = hooks.use_rerender_signal();
    hooks.use_spawn({
        let console = console.clone();
        move |_| {
            console.lock().unwrap().on_update(move || render_signal());

            move |_| {
                console.lock().unwrap().clear_update();
            }
        }
    });
    let (command, set_command) = hooks.use_state_with(|_| String::new());

    FocusRoot::el([WindowSized::el([with_rect(Dock::el([
        // text entry
        TextEditor::new(command, set_command.clone())
            .auto_focus()
            .placeholder(Some("Enter command..."))
            .on_submit({
                let console = console.clone();
                move |text| {
                    console.lock().unwrap().input(&text);
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
            FlowColumn::el({
                let console = console.lock().unwrap();
                console
                    .lines()
                    .iter()
                    .map(|m| Text::el(&m.text).with(color(), m.ty.into()))
                    .collect::<Vec<_>>()
            })
            .with_padding_even(4.0),
        )
        .with_background(vec4(0.0, 0.0, 0.0, 0.5))
        .with_default(docking_fill())
        .with_margin_even(STREET),
    ]))
    .with_background(vec4(0.0, 0.0, 0.0, 0.5))])
    .with_padding_even(20.)])
}
