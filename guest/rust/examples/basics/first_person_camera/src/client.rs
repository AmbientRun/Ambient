use ambient_api::prelude::*;

#[main]
fn main() {
    ambient_api::messages::Frame::subscribe(move |_| {
        let pressed = input::get();

        let mut displace = Vec2::ZERO;
        if pressed.keys.contains(&KeyCode::W) {
            displace.y -= 1.0;
        }
        if pressed.keys.contains(&KeyCode::S) {
            displace.y += 1.0;
        }
        if pressed.keys.contains(&KeyCode::A) {
            displace.x -= 1.0;
        }
        if pressed.keys.contains(&KeyCode::D) {
            displace.x += 1.0;
        }

        messages::Input::new(displace, pressed.mouse_position).send_server_unreliable();
    });
}
