use ambient_api::{
    components::core::{
        app::{main_scene, window_logical_size},
        camera::{orthographic_bottom, orthographic_left, orthographic_right, orthographic_top},
    },
    concepts::make_orthographic_camera,
    prelude::*,
};

mod constants;
use constants::*;

use components::track_audio_url;

#[main]
async fn main() {

    let url_from_server =
        entity::wait_for_component(entity::synchronized_resources(), track_audio_url())
            .await
            .unwrap();
    println!("url_from_server: {:?}", &url_from_server);

    // this is just to demo that you can load a sound from a url from the server
    let bgm_player = audio::AudioPlayer::new();
    bgm_player.set_amplitude(0.2);
    bgm_player.play(url_from_server);

    let camera_id = make_orthographic_camera()
        .with_default(main_scene())
        .spawn();

    ambient_api::messages::Frame::subscribe(move |_| {
        let input = input::get();
        let delta = input::get_delta().0;
        let mut direction = 0.0;

        if input.keys.contains(&KeyCode::Up) {
            direction += 1.0;
        }
        if input.keys.contains(&KeyCode::Down) {
            direction -= 1.0;
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
