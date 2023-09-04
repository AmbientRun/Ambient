use ambient_api::{
    core::{layout::components::space_between_items, messages::Frame},
    prelude::*,
};
use packages::this::{
    assets,
    components::{ball_ref, player_head_ref},
    messages::Input,
};

#[main]
fn main() {
    let mut cursor_lock = input::CursorLockGuard::new();

    Frame::subscribe(move |_| {
        let input = input::get();
        if !cursor_lock.auto_unlock_on_escape(&input) {
            return;
        }
        let mut displace = Vec2::ZERO;
        if input.keys.contains(&KeyCode::W) {
            displace.y -= 1.0;
        }
        if input.keys.contains(&KeyCode::S) {
            displace.y += 1.0;
        }
        if input.keys.contains(&KeyCode::A) {
            displace.x -= 1.0;
        }
        if input.keys.contains(&KeyCode::D) {
            displace.x += 1.0;
        }

        Input::new(displace, input.mouse_delta).send_server_unreliable();
    });
}
