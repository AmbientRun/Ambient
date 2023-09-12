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

use packages::this::{components::track_audio_url, messages::Input};

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

    let camera_id = OrthographicCamera {
        local_to_world: Mat4::IDENTITY,
        near: -1.,
        projection: Mat4::IDENTITY,
        projection_view: Mat4::IDENTITY,
        active_camera: 0.0,
        inv_local_to_world: Mat4::IDENTITY,
        orthographic: (),
        orthographic_left: -1.,
        orthographic_right: 1.,
        orthographic_top: 1.,
        orthographic_bottom: -1.,
        far: 1.,
        optional: OrthographicCameraOptional {
            main_scene: Some(()),
            ..default()
        },
    }
    .make()
    .spawn();

    Frame::subscribe(move |_| {
        let input = input::get();
        let mut direction = 0.0;

        if input.keys.contains(&KeyCode::Up) {
            direction += 1.0;
        }
        if input.keys.contains(&KeyCode::Down) {
            direction -= 1.0;
        }
        Input::new(direction).send_server_unreliable();
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
