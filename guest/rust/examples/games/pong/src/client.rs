use ambient_api::{
    components::core::{
        app::{main_scene, window_logical_size},
        camera::{orthographic_bottom, orthographic_left, orthographic_right, orthographic_top},
    },
    concepts::make_orthographic_camera,
    player::KeyCode,
    prelude::*,
};

mod constants;
use constants::*;

#[main]
fn main() {
    let mut bgm =
        audio::load(asset::url("assets/Kevin_MacLeod_8bit_Dungeon_Boss_ncs.ogg").unwrap());
    let mut ping = audio::load(asset::url("assets/ping.ogg").unwrap());
    let id = bgm.looping(true).volume(0.2).play();
    let mut is_playing = true;

    messages::Ping::subscribe(move |_, _| {
        ping.looping(false).volume(0.9).play();
    });

    let camera_id = make_orthographic_camera()
        .with_default(main_scene())
        .spawn();

    ambient_api::messages::Frame::subscribe(move |_| {
        let (delta, input) = player::get_raw_input_delta();
        let mut direction = 0.0;

        if input.keys.contains(&KeyCode::Up) {
            direction += 1.0;
        }
        if input.keys.contains(&KeyCode::Down) {
            direction -= 1.0;
        }

        if delta.keys.contains(&KeyCode::Key1) {
            bgm.volume(bgm.volume - 0.1);
        }

        if delta.keys.contains(&KeyCode::Key2) {
            bgm.volume(bgm.volume + 0.1);
        }

        if delta.keys.contains(&KeyCode::Key3) {
            if is_playing {
                bgm.stop();
            } else {
                bgm.play();
            }
            is_playing = !is_playing;
            // you can also use the id to stop the sound
            // id.stop();
        }

        messages::Input::new(direction).send_server_unreliable();
    });

    // Update camera so we have correct aspect ratio
    change_query(window_logical_size())
        .track_change(window_logical_size())
        .bind(move |windows| {
            for (_, window) in windows {
                let window = window.as_vec2();
                if window.x <= 0. || window.y <= 0. {
                    continue;
                }

                let x_boundary = X_BOUNDARY + SCREEN_PADDING;
                let y_boundary = Y_BOUNDARY + SCREEN_PADDING;
                let (left, right, top, bottom) = if window.x < window.y {
                    (
                        -x_boundary,
                        x_boundary,
                        y_boundary * window.y / window.x,
                        -y_boundary * window.y / window.x,
                    )
                } else {
                    (
                        -x_boundary * window.x / window.y,
                        x_boundary * window.x / window.y,
                        y_boundary,
                        -y_boundary,
                    )
                };
                entity::set_component(camera_id, orthographic_left(), left);
                entity::set_component(camera_id, orthographic_right(), right);
                entity::set_component(camera_id, orthographic_top(), top);
                entity::set_component(camera_id, orthographic_bottom(), bottom);
            }
        });
}
