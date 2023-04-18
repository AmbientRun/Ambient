use ambient_api::prelude::*;

#[main]
pub fn main() {
    let mut cursor_lock = input::CursorLockGuard::new(true);
    ambient_api::messages::Frame::subscribe(move |_| {
        let (delta, input) = input::get_delta();
        if !cursor_lock.auto_unlock_on_escape(&input) {
            return;
        }

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
