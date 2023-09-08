use ambient_api::{
    core::{
        app::components::{main_scene, name},
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        messages::Frame,
        player::components::is_player,
        transform::components::{local_to_parent, translation},
        ui::{components::focus, messages::FocusChanged},
    },
    entity::{add_child, get_component, mutate_component_with_default, resources, set_component},
    input::{set_cursor_lock, set_cursor_visible},
    prelude::*,
};

fn update() {
    let current_focus = get_component(resources(), focus()).unwrap_or_default();
    println!("FOCUS: {}", current_focus);
    if current_focus.is_empty() {
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
        let current_focus = get_component(resources(), focus()).unwrap_or_default();
        if current_focus.is_empty() {
            let (delta, input) = input::get_delta();
            if input.keys.contains(&KeyCode::Escape) {
                set_component(resources(), focus(), "Nothing".to_string());
                FocusChanged {}.send_local_broadcast(true);
            }
        }
    });
    FocusChanged::subscribe(|_, _| {
        update();
    });
    update();
}
