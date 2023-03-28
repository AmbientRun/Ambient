use ambient_api::{
    components::core::player::{player, user_id},
    prelude::*,
};

#[main]
pub async fn main() -> ResultEmpty {
    query(player()).build().each_frame(|ids| {
        for (id, _) in ids {
            let Some((delta, _)) = player::get_raw_input_delta(id) else { continue; };
            let Some(name) = entity::get_component(id, user_id()) else { continue; };

            if !delta.keys.is_empty() {
                println!("{name} pressed the keys {:?}", delta.keys);
            }
            if !delta.keys_released.is_empty() {
                println!("{name} released the keys {:?}", delta.keys_released);
            }
            if !delta.mouse_buttons.is_empty() {
                println!("{name} pressed the mouse buttons {:?}", delta.mouse_buttons);
            }
            if !delta.mouse_buttons_released.is_empty() {
                println!(
                    "{name} released the mouse buttons {:?}",
                    delta.mouse_buttons_released
                );
            }
            if delta.mouse_wheel != 0.0 {
                println!("{name} scrolled {}", delta.mouse_wheel);
            }
            if delta.mouse_position.length_squared() != 0.0 {
                println!("{name} moved their mouse by {}", delta.mouse_position);
            }
        }
    });

    OkEmpty
}
