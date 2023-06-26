#[allow(unused_imports)]
use ambient_api::{
    animation::{get_bone_by_bind_id, BindId},
    components::core::{model::model_loaded, prefab::prefab_from_url, transform::reset_scale},
    concepts::make_transformable,
    entity::{add_child, wait_for_component},
    prelude::*,
};

#[main]

pub fn main() {
    let mut last_shot = time();
    let mut is_shooting = false;

    ambient_api::messages::Frame::subscribe(move |_| {
        let (_delta, input) = input::get_delta();
        let mouse_delta = input.mouse_delta;
        let mut direction = Vec2::ZERO;
        let mut shoot = false;
        let mut walk = false;
        let mut jump = false;
        let mut duck = false;

        if input.keys.contains(&KeyCode::W) {
            direction.y -= 1.0;
        }
        if input.keys.contains(&KeyCode::S) {
            direction.y += 1.0;
        }
        if input.keys.contains(&KeyCode::A) {
            direction.x -= 1.0;
        }
        if input.keys.contains(&KeyCode::D) {
            direction.x += 1.0;
        }

        if input.keys.contains(&KeyCode::Space) {
            jump = true;
        }

        if input.keys.contains(&KeyCode::LControl) {
            duck = true;
        }

        if input.keys.contains(&KeyCode::LShift) {
            walk = true;
        }

        if input.mouse_buttons.contains(&MouseButton::Left) {
            if is_shooting {
                if time() - last_shot > Duration::from_millis(1000) {
                    shoot = true;

                    last_shot = time();
                }
            } else {
                shoot = true;
                is_shooting = true;
                last_shot = time();
            }
        } else {
            is_shooting = false;
        }

        // if shoot {

        // }

        // let player_id = player::get_local();
        // if shoot {
        //     // println!("shoot");
        //     let cam = entity::get_component(player_id, components::player_head_ref()).unwrap();
        //     let window_size =
        //         entity::get_component(entity::resources(), window_logical_size()).unwrap();
        //     let ray = camera::screen_position_to_world_ray(
        //         cam,
        //         vec2(window_size.x as f32 / 2., window_size.y as f32 / 2.),
        //     );
        //     messages::Ray {
        //         ray_origin: ray.origin,
        //         ray_dir: ray.dir,
        //         source: player_id,
        //         type_action: 0,
        //     }
        //     .send_server_unreliable();
        // }

        messages::Input {
            direction,
            mouse_delta,
            shoot,
            walk,
            jump,
            duck,
        }
        .send_server_unreliable();
    });
}
