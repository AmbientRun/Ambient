use ambient_api::{
    core::{
        messages::{Frame, WindowCursorLockChange},
        transform::components::translation,
        ui::{components::focusable, messages::FocusChanged},
    },
    input::{is_game_focused, set_cursor_lock, set_cursor_visible, GAME_FOCUS_ID},
    prelude::*,
};

fn update() {
    if is_game_focused() {
        set_cursor_lock(true);
        set_cursor_visible(false);
    } else {
        set_cursor_lock(false);
        set_cursor_visible(true);
    }
}

pub mod packages;

#[main]
pub fn main() {
    WindowSized::el([])
        .init(translation(), vec3(0., 0., 1.1))
        .with_clickarea()
        .el()
        .with(focusable(), GAME_FOCUS_ID.to_string())
        .spawn_interactive();

    Frame::subscribe(move |_| {
        if is_game_focused() {
            let input = input::get();
            if input.keys.contains(&KeyCode::Escape) {
                input::set_focus("Nothing");
            }
        }
    });
    WindowCursorLockChange::subscribe(|msg| {
        if is_game_focused() && !msg.locked {
            input::set_focus("Nothing");
        }
    });
    FocusChanged::subscribe(|_, _| {
        update();
    });

    update();
}
