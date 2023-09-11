use ambient_api::{
    core::{messages::Frame, physics::components::linear_velocity},
    input::is_game_focused,
    prelude::*,
};
use packages::this::{
    assets,
    messages::{Bonk, Hit, Input},
};

#[main]
fn main() {
    Frame::subscribe(move |_| {
        if !is_game_focused() {
            return;
        }
        let (delta, _input) = input::get_delta();

        Input {
            camera_rotation: delta.mouse_position,
            camera_zoom: delta.mouse_wheel,
            shoot: delta.mouse_buttons.contains(&MouseButton::Left),
        }
        .send_server_unreliable();
    });

    let ball_hit_player = audio::AudioPlayer::new();
    let ball_drop_player = audio::AudioPlayer::new();
    Hit::subscribe(move |_ctx, data| {
        let ball = data.ball;
        let vel = entity::get_component(ball, linear_velocity()).unwrap_or_default();
        let mut amp = (vel.x.abs() / 5.0).powf(2.0)
            + (vel.y.abs() / 5.0).powf(2.0)
            + (vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        ball_hit_player.set_amplitude(amp);
        ball_hit_player.play(assets::url("ball-hit.ogg"));
    });

    Bonk::subscribe(move |_ctx, data| {
        let ball = data.ball;
        let vel = entity::get_component(ball, linear_velocity()).unwrap_or_default();
        let mut amp = (vel.x.abs() / 5.0).powf(2.0)
            + (vel.y.abs() / 5.0).powf(2.0)
            + (vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        ball_drop_player.set_amplitude(amp);
        ball_drop_player.play(assets::url("ball-drop.ogg"));
    });
}
