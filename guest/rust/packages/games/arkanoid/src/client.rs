use ambient_api::{
    core::{
        app::components::window_logical_size,
        camera::{
            components::{
                orthographic_bottom, orthographic_left, orthographic_right, orthographic_top,
            },
            concepts::{OrthographicCamera, OrthographicCameraOptional},
        },
        messages::Frame,
    },
    prelude::*,
};

mod constants;
use constants::*;

use packages::this::{
    components::track_audio_url,
    messages::{Input, Ping},
};

#[main]
async fn main() {
    let url_from_server =
        entity::wait_for_component(entity::synchronized_resources(), track_audio_url())
            .await
            .unwrap();
    println!("url_from_server: {:?}", &url_from_server);

    let bgm_player = audio::AudioPlayer::new();
    bgm_player.set_amplitude(0.2);

    Ping::subscribe(move |_ctx, _data| {
        bgm_player.play(url_from_server.clone());
    });

    let camera_id = OrthographicCamera {
        optional: OrthographicCameraOptional {
            main_scene: Some(()),
            ..default()
        },
        ..OrthographicCamera::suggested()
    }
    .spawn();

    Frame::subscribe(move |_| {
        let input = input::get();
        let mut direction = 0.0;

        if input.keys.contains(&KeyCode::Right) {
            direction += 1.0;
        }
        if input.keys.contains(&KeyCode::Left) {
            direction -= 1.0;
        }

        Input {
            direction,
            start: input.keys.contains(&KeyCode::Space),
        }
        .send_server_reliable();
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
