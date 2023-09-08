use ambient_api::{core::player::components::is_player, prelude::*};

use packages::afps_schema::{
    components::{player_name, player_shooting_status, player_zoomed},
    messages::{Input, Shoot},
};
use packages::unit_schema::components::vertical_velocity;

#[main]
pub fn main() {
    spawn_query(is_player()).bind(|results| {
        for (id, ()) in results {
            run_async(async move {
                if entity::wait_for_component(id, player_name())
                    .await
                    .is_none()
                {
                    // entity deleted
                    return;
                }
                entity::add_component(id, player_zoomed(), false);
                entity::add_component(id, vertical_velocity(), 0.0);
            });
        }
    });

    // let mut last_walk = game_time();
    Input::subscribe(move |ctx, msg| {
        // receive movement and send this for further processing
        let player_id = ctx.client_entity_id();
        if player_id.is_none() {
            return;
        }
        let player_id = player_id.unwrap();
        // let direction = msg.direction;

        // if direction != Vec2::ZERO {
        //     let dur = if msg.running {
        //         Duration::from_millis(400)
        //     } else {
        //         Duration::from_millis(600)
        //     };
        //     let is_jumping = entity::get_component(player_id, vertical_velocity()).unwrap_or(0.0);
        //     if is_jumping <= 0.0 && game_time() - last_walk > dur {
        //         last_walk = game_time();
        //         if msg.running {
        //             FootOnGround { source: player_id }.send_local_broadcast(false);
        //         } // keep silent when walking
        //     }
        // }

        // // temporary fix pos for shooting
        // if !msg.is_shooting {
        //     entity::add_component(player_id, run_direction(), direction);
        // } else {
        //     entity::add_component(player_id, run_direction(), Vec2::ZERO);
        // }

        entity::add_component(player_id, player_shooting_status(), msg.is_shooting);

        if msg.toggle_zoom {
            entity::mutate_component(player_id, player_zoomed(), |v| *v = !*v);
        }

        if msg.shoot {
            Shoot {
                ray_origin: msg.ray_origin,
                ray_dir: msg.ray_dir,
                source: player_id,
            }
            .send_local_broadcast(false);
        }
    });
}
