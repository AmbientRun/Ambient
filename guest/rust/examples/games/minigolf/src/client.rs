use ambient_api::{
    core::{messages::Frame, physics::components::linear_velocity},
    prelude::*,
};
use ambient_example_minigolf::messages::{Bonk, Hit, Input};

#[main]
fn main() {
    let mut cursor_lock = input::CursorLockGuard::new();
    Frame::subscribe(move |_| {
        let (delta, input) = input::get_delta();
        if !cursor_lock.auto_unlock_on_escape(&input) {
            return;
        }

        Input {
            camera_rotation: delta.mouse_position,
            camera_zoom: delta.mouse_wheel,
            shoot: delta.mouse_buttons.contains(&MouseButton::Left),
        }
        .send_server_unreliable();
    });

    let ball_hit_player = audio::AudioPlayer::new();
    let ball_drop_player = audio::AudioPlayer::new();
    Hit::subscribe(move |_source, data| {
        let ball = data.ball;
        let vel = entity::get_component(ball, linear_velocity()).unwrap_or_default();
        let mut amp = (vel.x.abs() / 5.0).powf(2.0)
            + (vel.y.abs() / 5.0).powf(2.0)
            + (vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        ball_hit_player.set_amplitude(amp);
        ball_hit_player.play(asset::url("assets/ball-hit.ogg").unwrap());
    });

    Bonk::subscribe(move |_source, data| {
        let ball = data.ball;
        let vel = entity::get_component(ball, linear_velocity()).unwrap_or_default();
        let mut amp = (vel.x.abs() / 5.0).powf(2.0)
            + (vel.y.abs() / 5.0).powf(2.0)
            + (vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        ball_drop_player.set_amplitude(amp);
        ball_drop_player.play(asset::url("assets/ball-drop.ogg").unwrap());
    });
}
