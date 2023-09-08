use ambient_api::{
    core::{
        messages::Frame,
        ui::{components::focus, messages::FocusChanged},
    },
    entity::{resources, set_component},
    input::{is_game_focused, set_cursor_lock, set_cursor_visible},
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

#[main]
pub fn main() {
    Frame::subscribe(move |_| {
        if is_game_focused() {
            let (_, input) = input::get_delta();
            if input.keys.contains(&KeyCode::Escape) {
                set_component(resources(), focus(), "Nothing".to_string());
                FocusChanged {
                    from_external: false,
                    focus: "Nothing".to_string(),
                }
                .send_local_broadcast(true);
            }
        }
    });
    FocusChanged::subscribe(|_, _| {
        update();
    });
    update();
}
