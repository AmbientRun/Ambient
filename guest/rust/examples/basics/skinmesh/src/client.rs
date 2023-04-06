use ambient_api::{player::KeyCode, prelude::*};

#[main]
pub fn main() {
    ambient_api::messages::Frame::subscribe(move |_| {
        let (delta, _) = player::get_raw_input_delta();

        if delta.keys.contains(&KeyCode::Key1) {
            messages::SetController::new(1u32).send_server_reliable();
        }

        if delta.keys.contains(&KeyCode::Key2) {
            messages::SetController::new(2u32).send_server_reliable();
        }

        if delta.keys.contains(&KeyCode::Key3) {
            messages::SetController::new(3u32).send_server_reliable();
        }
    });
}
