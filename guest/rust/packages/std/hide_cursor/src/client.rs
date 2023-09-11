use ambient_api::{
    core::{
        messages::Frame,
        transform::components::translation,
        ui::{
            components::{focus, focusable},
            messages::FocusChanged,
        },
    },
    entity::{resources, set_component},
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
