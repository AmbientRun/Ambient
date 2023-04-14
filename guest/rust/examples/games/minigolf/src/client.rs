use ambient_api::{
    player::MouseButton,
    prelude::*,
    components::core::{
        physics::{
            linear_velocity
        }
    }
};

#[main]
fn main() {
    ambient_api::messages::Frame::subscribe(move |_| {
        let (delta, input) = player::get_raw_input_delta();

        let camera_rotation = if input.mouse_buttons.contains(&MouseButton::Right) {
            delta.mouse_position
        } else {
            Vec2::ZERO
        };

        let camera_zoom = delta.mouse_wheel;
        let shoot = delta.mouse_buttons.contains(&MouseButton::Left);

        messages::Input::new(camera_rotation, camera_zoom, shoot).send_server_unreliable();
    });

    let mut ballhit = audio::load(asset::url("assets/ball-hit.ogg").unwrap());

    let mut balldrop = audio::load(asset::url("assets/ball-drop.ogg").unwrap());

    messages::Hit::subscribe(move |_source, data| {
        let ball = data.ball;
        let vel = entity::get_component(ball, linear_velocity()).unwrap();
        let mut amp = (vel.x.abs() / 5.0).powf(2.0)
        + (vel.y.abs() / 5.0).powf(2.0)
        + (vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        ballhit.looping(false).scale(amp).play();
    });

    messages::Bonk::subscribe(move |_source, data| {
        let ball = data.ball;
        let vel = entity::get_component(ball, linear_velocity()).unwrap();
        let mut amp = (vel.x.abs() / 5.0).powf(2.0)
        + (vel.y.abs() / 5.0).powf(2.0)
        + (vel.z.abs() / 5.0).powf(2.0);
        amp = amp.sqrt().clamp(0.0, 1.0);
        amp = amp * amp;
        balldrop.looping(false).scale(amp).play();
    });
}
