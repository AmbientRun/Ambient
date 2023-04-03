use ambient_api::{player::KeyCode, prelude::*};

#[main]
fn main() {
    ambient_api::messages::Frame::subscribe(move |_, _| {
        let (delta, pressed) = player::get_raw_input_delta();

        let mut displace = Vec2::ZERO;
        if pressed.keys.contains(&KeyCode::W) {
            displace.x += 1.0;
        }
        if pressed.keys.contains(&KeyCode::S) {
            displace.x -= 1.0;
        }
        if pressed.keys.contains(&KeyCode::A) {
            displace.y -= 1.0;
        }
        if pressed.keys.contains(&KeyCode::D) {
            displace.y += 1.0;
        }

        messages::Input::new(displace, delta.mouse_position.x).send(Target::ServerReliable);
    });
}
