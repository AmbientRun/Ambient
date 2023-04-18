use ambient_api::{components::core::physics::linear_velocity, prelude::*};

#[main]
fn main() {
    let mut cursor_lock = input::CursorLockGuard::new(true);
    ambient_api::messages::Frame::subscribe(move |_| {
        let (delta, input) = input::get_delta();
        if !cursor_lock.auto_unlock_on_escape(&input) {
            return;
        }

        messages::Input {
            camera_rotation: delta.mouse_position,
            camera_zoom: delta.mouse_wheel,
            shoot: delta.mouse_buttons.contains(&MouseButton::Left),
        }
        .send_server_unreliable();
    });

    let ballhit = audio::load(asset::url("assets/ball-hit.ogg").unwrap());

    let balldrop = audio::load(asset::url("assets/ball-drop.ogg").unwrap());

    messages::Hit::subscribe(move |_source, data| {
        let ball = data.ball;
        let vel = entity::get_component(ball, linear_velocity()).unwrap();
        let mut amp = (vel.x.abs() / 5.0).powf(2.0)
            + (vel.y.abs() / 5.0).powf(2.0)
            + (vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        ballhit.looping(false).volume(amp).play();
    });

    messages::Bonk::subscribe(move |_source, data| {
        let ball = data.ball;
        let vel = entity::get_component(ball, linear_velocity()).unwrap();
        let mut amp = (vel.x.abs() / 5.0).powf(2.0)
            + (vel.y.abs() / 5.0).powf(2.0)
            + (vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        balldrop.looping(false).volume(amp).play();
    });
}
