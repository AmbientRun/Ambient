use ambient_api::{
    components::core::{player::player, transform::rotation},
    prelude::*,
};

#[main]
pub fn main() {
    // spawn_query(player()).bind(|results| {
    //     for (id, ()) in results {

    //     }
    // });
    spawn_query(player()).bind(|results| {
        println!("___player movement triggered___");
        for (id, ()) in results {
            run_async(async move {
                entity::wait_for_component(id, components::player_name()).await;
                entity::add_component(id, components::player_yaw(), 0.0);
                entity::add_component(id, components::player_pitch(), 0.0);
                entity::add_component(id, components::player_zoomed(), false);
                entity::add_component(id, components::player_vspeed(), 0.0);
            });
        }
    });

    let mut last_walk = game_time();
    messages::Input::subscribe(move |source, msg| {
        // receive movement and send this for further processing
        let player_id = source.client_entity_id();
        if player_id.is_none() {
            return;
        }
        let player_id = player_id.unwrap();
        let direction = msg.direction;

        if direction != Vec2::ZERO {
            if game_time() - last_walk > Duration::from_millis(700) {
                last_walk = game_time();
                messages::FootOnGround { source: player_id }.send_local_broadcast(false);
            }
        }

        if msg.jump {
            println!("___jump triggered___");
            // components::player_vspeed(),
            entity::add_component(player_id, components::player_vspeed(), 0.6);
        }

        // temporary fix pos for shooting
        if !msg.is_shooting {
            entity::add_component(player_id, components::player_direction(), direction);
        } else {
            entity::add_component(player_id, components::player_direction(), Vec2::ZERO);
        }

        entity::add_component(
            player_id,
            components::player_shooting_status(),
            msg.is_shooting,
        );

        if msg.toggle_zoom {
            entity::mutate_component(player_id, components::player_zoomed(), |v| *v = !*v);
        }

        let yaw = entity::mutate_component(player_id, components::player_yaw(), |yaw| {
            *yaw = (*yaw + msg.mouse_delta.x * 0.01) % std::f32::consts::TAU;
        })
        .unwrap_or_default();

        entity::set_component(player_id, rotation(), Quat::from_rotation_z(yaw));

        let pitch = entity::mutate_component(player_id, components::player_pitch(), |pitch| {
            *pitch = (*pitch + msg.mouse_delta.y * 0.01)
                .clamp(-std::f32::consts::FRAC_PI_3, std::f32::consts::FRAC_PI_3);
        })
        .unwrap_or_default();

        if let Some(cam_id) = entity::get_component(player_id, components::player_cam_ref()) {
            entity::set_component(
                cam_id,
                rotation(),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2 + pitch),
            );
        }

        if msg.shoot {
            messages::Shoot {
                ray_origin: msg.ray_origin,
                ray_dir: msg.ray_dir,
                source: player_id,
            }
            .send_local_broadcast(false);
        }
    });

    query((
        player(),
        components::player_direction(),
        rotation(),
        components::player_vspeed(),
    ))
    .each_frame(move |list| {
        for (player_id, (_, direction, rot, vspeed)) in list {
            let speed = vec2(0.04, 0.06);
            let displace = rot * (direction.normalize_or_zero() * speed).extend(vspeed);
            let collision = physics::move_character(player_id, displace, 0.01, delta_time());
            if collision.down {
                entity::set_component(player_id, components::player_vspeed(), 0.0);
            } else {
                entity::mutate_component(player_id, components::player_vspeed(), |vspeed| {
                    *vspeed -= 3.0 * delta_time(); // 1/60 second for example
                });
            }
        }
    });
}
