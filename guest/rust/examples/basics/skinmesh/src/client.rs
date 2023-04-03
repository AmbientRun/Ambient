use ambient_api::{
    message::{MessageExt, Target},
    player::KeyCode,
    prelude::*,
};

#[main]
pub fn main() {
    on(event::FRAME, |_| {
        let (delta, _) = player::get_raw_input_delta();

        if delta.keys.contains(&KeyCode::Key1) {
            messages::SetController::new(1u32).send(Target::RemoteReliable);
        }

        if delta.keys.contains(&KeyCode::Key2) {
            messages::SetController::new(2u32).send(Target::RemoteReliable);
        }

        if delta.keys.contains(&KeyCode::Key3) {
            messages::SetController::new(3u32).send(Target::RemoteReliable);
        }
    });
}
