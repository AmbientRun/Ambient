use ambient_api::{core::messages::Frame, input::is_game_focused, prelude::*};

#[main]
pub fn main() {
    Frame::subscribe(move |_| {
        if !is_game_focused() {
            return;
        }
        let (delta, _input) = input::get_delta();

        if !delta.keys.is_empty() {
            println!("Pressed the keys {:?}", delta.keys);
        }
        if !delta.keys_released.is_empty() {
            println!("Released the keys {:?}", delta.keys_released);
        }
        if !delta.mouse_buttons.is_empty() {
            println!("Pressed the mouse buttons {:?}", delta.mouse_buttons);
        }
        if !delta.mouse_buttons_released.is_empty() {
            println!(
                "Released the mouse buttons {:?}",
                delta.mouse_buttons_released
            );
        }
        if delta.mouse_wheel != 0.0 {
            println!("Scrolled {}", delta.mouse_wheel);
        }
        if delta.mouse_position.length_squared() != 0.0 {
            println!("Moved their mouse by {}", delta.mouse_position);
        }
    });
}
