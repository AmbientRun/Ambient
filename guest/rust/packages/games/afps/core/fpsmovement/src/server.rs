use ambient_api::{
    core::{player::components::is_player, transform::components::rotation},
    prelude::*,
};

use packages::unit_schema::components::{jumping, run_direction, running, vertical_velocity};
use packages::{
    afps_schema::{
        components::{
            player_name, player_pitch, player_shooting_status, player_yaw, player_zoomed,
        },
        messages::{FootOnGround, Input, Shoot},
    },
    unit_schema::components::head_ref,
};
use std::f32::consts::PI;

const INIT_JUMP_VSPEED: f32 = 0.10;

#[main]
pub fn main() {
    spawn_query(is_player()).bind(|results| {
        for (id, ()) in results {
            run_async(async move {
                if entity::wait_for_component(id, player_name()).await.is_none() {
                    // entity deleted
                    return;
                }
                entity::add_component(id, player_yaw(), 0.0);
                entity::add_component(id, player_pitch(), 0.0);
                entity::add_component(id, player_zoomed(), false);
                entity::add_component(id, vertical_velocity(), 0.0);
            });
        }
    });

    let mut last_walk = game_time();
    Input::subscribe(move |ctx, msg| {
        // receive movement and send this for further processing
        let player_id = ctx.client_entity_id();
        if player_id.is_none() {
            return;
        }
        let player_id = player_id.unwrap();
        let direction = msg.direction;

        if direction != Vec2::ZERO {
            let dur = if msg.running {
                Duration::from_millis(400)
            } else {
                Duration::from_millis(600)
            };
            let is_jumping = entity::get_component(player_id, vertical_velocity()).unwrap_or(0.0);
            if is_jumping <= 0.0 && game_time() - last_walk > dur {
                last_walk = game_time();
                if msg.running {
                    FootOnGround { source: player_id }.send_local_broadcast(false);
                } // keep silent when walking
            }
        }

        if msg.jump {
            entity::add_component(player_id, jumping(), true);
            entity::add_component(player_id, vertical_velocity(), INIT_JUMP_VSPEED);
        }

        if msg.running {
            entity::add_component(player_id, running(), true);
        } else {
            entity::add_component(player_id, running(), false);
        }

        // temporary fix pos for shooting
        if !msg.is_shooting {
            entity::add_component(player_id, run_direction(), direction);
        } else {
            entity::add_component(player_id, run_direction(), Vec2::ZERO);
        }

        entity::add_component(player_id, player_shooting_status(), msg.is_shooting);

        if msg.toggle_zoom {
            entity::mutate_component(player_id, player_zoomed(), |v| *v = !*v);
        }

        let yaw = entity::mutate_component(player_id, player_yaw(), |yaw| {
            *yaw = (*yaw + msg.mouse_delta.x * 0.01) % std::f32::consts::TAU;
        })
        .unwrap_or_default();

        entity::set_component(player_id, rotation(), Quat::from_rotation_z(yaw));

        let pitch = entity::mutate_component(player_id, player_pitch(), |pitch| {
            *pitch = (*pitch + msg.mouse_delta.y * 0.01)
                .clamp(-std::f32::consts::FRAC_PI_3, std::f32::consts::FRAC_PI_3);
        })
        .unwrap_or_default();

        if let Some(head_id) = entity::get_component(player_id, head_ref()) {
            entity::set_component(
                head_id,
                rotation(),
                Quat::from_rotation_y(pitch)
                    * Quat::from_rotation_z(PI / 2.)
                    * Quat::from_rotation_x(PI / 2.),
            );
        }

        if msg.shoot {
            Shoot {
                ray_origin: msg.ray_origin,
                ray_dir: msg.ray_dir,
                source: player_id,
            }
            .send_local_broadcast(false);

            let _recoil = entity::mutate_component(player_id, player_pitch(), |pitch| {
                let recoil = random::<f32>() * 0.01;
                *pitch -= recoil;
            })
            .unwrap_or_default();
        }
    });
}
