use std::{cell::RefCell, rc::Rc};

use ambient_api::{components::core::app::window_logical_size, input::CursorLockGuard, prelude::*};

#[main]
pub async fn main() {
    ambient_api::messages::Frame::subscribe(move |_| {
        let (delta, _input) = input::get_delta();
        if delta.keys.contains(&KeyCode::T) {
            let input = input::get();

            let player_id = player::get_local();
            let cam = entity::get_component(player_id, components::player_cam_ref());

            let cam = if let Some(cam) = cam {
                cam
            } else {
                return;
            };
            // let ray = camera::clip_position_to_world_ray(cam, Vec2::ZERO);

            let window_size =
                entity::get_component(entity::resources(), window_logical_size()).unwrap();
            let ray = camera::screen_position_to_world_ray(
                cam,
                vec2(window_size.x as f32 / 2., window_size.y as f32 / 2.),
            );

            // Send screen ray to server
            messages::Spraypaint {
                origin: ray.origin,
                dir: ray.dir,
            }
            .send_server_unreliable();
            println!("Spray paint ray sent ");
        }
    });
}
