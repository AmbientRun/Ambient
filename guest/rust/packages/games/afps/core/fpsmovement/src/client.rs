use ambient_api::{
    core::{app::components::window_logical_size, messages::Frame},
    input::is_game_focused,
    prelude::*,
};

use packages::afps_schema::{
    components::{hit_freeze, player_cam_ref},
    messages::Input,
};

#[main]
pub async fn main() {
    let mut last_shot = game_time();
    let mut is_shooting = false;

    Frame::subscribe(move |_| {
        if !is_game_focused() {
            return;
        }

        let (delta, input) = input::get_delta();
        let mouse_delta = input.mouse_delta;
        let mut shoot = false;

        if input.mouse_buttons.contains(&MouseButton::Left) {
            if is_shooting {
                if game_time() - last_shot > Duration::from_millis(200) {
                    shoot = true;
                    last_shot = game_time();
                }
            } else {
                shoot = true;
                is_shooting = true;
                last_shot = game_time();
            }
        } else {
            is_shooting = false;
        }

        let toggle_zoom = delta.mouse_buttons.contains(&MouseButton::Right);

        let player_id = player::get_local();
        let hit_freeze_factor = entity::get_component(player_id, hit_freeze()).unwrap_or(0);
        if hit_freeze_factor > 0 {
            entity::set_component(player_id, hit_freeze(), hit_freeze_factor - 1);
            return;
        }
        let cam = entity::get_component(player_id, player_cam_ref());
        if cam.is_none() {
            return;
        }

        let cam = cam.unwrap();
        let window_size =
            entity::get_component(entity::resources(), window_logical_size()).unwrap();
        let ray = camera::screen_position_to_world_ray(
            cam,
            vec2(window_size.x as f32 / 2., window_size.y as f32 / 2.),
        );

        Input {
            mouse_delta,
            shoot,
            toggle_zoom,
            is_shooting,
            ray_origin: ray.origin,
            ray_dir: ray.dir,
        }
        .send_server_unreliable();
    });
}
