use ambient_api::{
    core::{app::components::window_logical_size, messages::Frame},
    prelude::*,
};

use afps_schema::components::player_cam_ref;
use afps_spraypaint::messages::Spraypaint;

#[main]
pub async fn main() {
    Frame::subscribe(move |_| {
        let (delta, _input) = input::get_delta();
        if delta.keys.contains(&KeyCode::T) {
            let player_id = player::get_local();
            let cam = entity::get_component(player_id, player_cam_ref());

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
            Spraypaint {
                origin: ray.origin,
                dir: ray.dir,
            }
            .send_server_unreliable();
            println!("Spray paint ray sent ");
        }
    });
}
