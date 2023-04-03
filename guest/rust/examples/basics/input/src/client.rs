use ambient_api::{message::MessageExt, prelude::*};

#[main]
pub fn main() {
    ambient_api::messages::Frame::subscribe(move |_, _| {
        let (delta, _) = player::get_raw_input_delta();

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
