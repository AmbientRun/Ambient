use ambient_api::{
    message::client::{MessageExt, Target},
    prelude::*,
};

#[main]
fn main() {
    on(event::FRAME, |_| {
        let (delta, _) = player::get_raw_input_delta();

        fn set_fog_density(val: f32) {
            messages::SetFogDensity::new(val).send(Target::RemoteReliable);
        }

        fn set_fog_height_falloff(val: f32) {
            messages::SetFogHeightFalloff::new(val).send(Target::RemoteReliable);
        }

        if delta.keys.contains(&player::KeyCode::Key1) {
            set_fog_density(1.);
        }
        if delta.keys.contains(&player::KeyCode::Key2) {
            set_fog_density(0.1);
        }
        if delta.keys.contains(&player::KeyCode::Key3) {
            set_fog_density(0.01);
        }
        if delta.keys.contains(&player::KeyCode::Key4) {
            set_fog_density(0.0);
        }

        if delta.keys.contains(&player::KeyCode::Q) {
            set_fog_height_falloff(1.);
        }
        if delta.keys.contains(&player::KeyCode::W) {
            set_fog_height_falloff(0.1);
        }
        if delta.keys.contains(&player::KeyCode::E) {
            set_fog_height_falloff(0.01);
        }
        if delta.keys.contains(&player::KeyCode::R) {
            set_fog_height_falloff(0.0);
        }
    });
}
