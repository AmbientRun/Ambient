// TODO: this should vary based on the game type

use std::f32::consts::PI;

use ambient_api::{
    core::{
        physics::components::{
            angular_velocity, cube_collider, dynamic, linear_velocity, physics_controlled,
        },
        player::components::is_player,
        primitives::components::cube,
        rendering::components::{cast_shadows, color},
        transform::{
            components::{rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use packages::afps_schema::{
    components::{
        self, heal_timeout, hit_freeze, player_deathcount, player_killcount, player_name,
        player_team,
    },
    messages::{Explosion, Shoot},
};
use packages::unit_schema::components::{health, vertical_velocity};

#[main]
pub fn main() {
    spawn_query(is_player()).bind(|results| {
        for (id, ()) in results {
            run_async(async move {
                if entity::wait_for_component(id, player_name()).await.is_none() {
                    // entity deleted
                    return;
                }
                entity::add_component(id, health(), 100.);
                entity::add_component(id, hit_freeze(), 0);
                entity::add_component(id, player_killcount(), 0);
                entity::add_component(id, player_deathcount(), 0);
                entity::add_component(id, heal_timeout(), 0);
            });
        }
    });

    Shoot::subscribe(move |_ctx, msg| {
        let result = physics::raycast_first(msg.ray_origin, msg.ray_dir, None);

        if let Some(hit) = result {
            // Laser gun, not used
            // run_async(async move {
            //     let laser_length = (hit.position - msg.ray_origin).length();
            //     let ray_direction = (hit.position - msg.ray_origin).normalize();
            //     let up = vec3(0.0, 0.0, 1.0);

            //     let right = up.cross(ray_direction).normalize();
            //     let up_direction = ray_direction.cross(right).normalize();

            //     let rot_matrix = Mat3::from_cols(right, up_direction, ray_direction);
            //     let rotation_quat = Quat::from_mat3(&rot_matrix);
            //     println!("laser length: {}", laser_length);
            //     let laser_center = (hit.position + msg.ray_origin) / 2.0;
            //     let laser = Entity::new()
            //         .with_merge(make_transformable())
            //         .with(cube(), ())
            //         .with(scale(), vec3(0.01, 0.01, laser_length * 0.6))
            //         .with(translation(), laser_center)
            //         .with(rotation(), rotation_quat)
            //         .with(color(), vec4(0.5, 0.4, 0.7, 0.8))
            //         .spawn();
            //     sleep(0.1).await;
            //     entity::despawn(laser);
            // });

            // TODO: just to test death anim
            // if hit.entity == msg.source {
            //     eprintln!("self hit");
            //     return;
            // }

            if entity::has_component(hit.entity, player_team()) {
                // let pos = entity::get_component(hit.entity, translation()).unwrap();
                let pos = hit.position;
                Explosion { pos }.send_local_broadcast(false);
                let c = entity::get_component(hit.entity, color()).unwrap();
                entity::despawn(hit.entity);
                run_async(async move {
                    for _ in 0..40 {
                        let max_linear_velocity = 2.5;
                        let max_angular_velocity = 360.0f32.to_radians();
                        let new_linear_velocity =
                            (random::<Vec3>() - 0.5) * 2. * max_linear_velocity;
                        let new_angular_velocity =
                            (random::<Vec3>() - 0.5) * 2. * max_angular_velocity;
                        let pos = pos + random::<Vec3>() * 6.0 - 3.0;

                        let size = random::<Vec3>() * 0.3;
                        let rot = Quat::from_rotation_y(random::<f32>() * PI)
                            * Quat::from_rotation_x(random::<f32>() * PI);
                        Entity::new()
                            .with_merge(make_transformable())
                            .with(cube(), ())
                            .with(rotation(), rot)
                            .with(physics_controlled(), ())
                            .with(cast_shadows(), ())
                            .with(linear_velocity(), new_linear_velocity)
                            .with(angular_velocity(), new_angular_velocity)
                            // .with(linear_velocity(), vec3(0.0, 0.0, 10.0)) //random::<Vec3>() * 20.0 - 10.0)
                            // .with(angular_velocity(), random::<Vec3>() * 1.0)
                            .with(cube_collider(), Vec3::ONE)
                            .with(dynamic(), true)
                            .with(scale(), random::<Vec3>() * size * 2.0)
                            .with(translation(), pos)
                            .with(color(), c)
                            .spawn();
                    }
                });
            }

            if let Some(old_health) = entity::get_component(hit.entity, health()) {
                if old_health <= 0. {
                    return;
                }

                let hit_back_dir = (msg.ray_origin - hit.position).normalize();
                let displace = hit_back_dir * -0.1;
                physics::move_character(hit.entity, displace, 0.001, delta_time());

                // rotation
                let forward = (hit.position - msg.ray_origin + random::<Vec3>() * 0.01).normalize();

                let forward_flat = vec3(forward.x, forward.y, 0.0).normalize();
                let rot = Quat::from_rotation_arc(vec3(0.0, 1.0, 0.0), forward_flat);

                entity::set_component(hit.entity, rotation(), rot);

                entity::set_component(hit.entity, vertical_velocity(), 0.04);

                let new_health = (old_health - 30.).max(0.);
                entity::set_component(hit.entity, health(), new_health);

                if old_health > 0. && new_health <= 0. {
                    println!("player dead, waiting for respawn");
                    // 114 is the death anim frame count
                    entity::set_component(hit.entity, components::hit_freeze(), 180);
                    entity::mutate_component(msg.source, components::player_killcount(), |count| {
                        *count += 1;
                    });
                    entity::mutate_component(
                        hit.entity,
                        components::player_deathcount(),
                        |count| {
                            *count += 1;
                        },
                    );

                    if entity::has_component(
                        entity::synchronized_resources(),
                        components::kill_log(),
                    ) {
                        entity::mutate_component(
                            entity::synchronized_resources(),
                            components::kill_log(),
                            |v| {
                                v.push(format!(
                                    "[{}] \u{e231} \u{f061} [{}]",
                                    entity::get_component(msg.source, components::player_name())
                                        .unwrap_or("unknown".to_string()),
                                    entity::get_component(hit.entity, components::player_name())
                                        .unwrap_or("unknown".to_string()),
                                ));
                            },
                        );
                        remove_last_history();
                    } else {
                        entity::add_component(
                            entity::synchronized_resources(),
                            components::kill_log(),
                            vec![format!(
                                "[{}] \u{e231} \u{f061} [{}]",
                                entity::get_component(msg.source, components::player_name())
                                    .unwrap_or("unknown".to_string()),
                                entity::get_component(hit.entity, components::player_name())
                                    .unwrap_or("unknown".to_string()),
                            )],
                        );
                        remove_last_history();
                    }

                    // TODO: wait for anim msg to respawn
                    run_async(async move {
                        sleep(3.).await;

                        if !entity::exists(hit.entity) {
                            return;
                        }

                        entity::set_component(
                            hit.entity,
                            translation(),
                            vec3(random::<f32>() * 10.0, random::<f32>() * 60.0 - 30., 2.0),
                        );
                        entity::set_component(hit.entity, health(), 100.);
                        entity::set_component(hit.entity, hit_freeze(), 0);
                    });
                } else {
                    entity::set_component(hit.entity, hit_freeze(), 20);
                    entity::set_component(hit.entity, heal_timeout(), 150);
                }
            }
        }
    });

    // change_query(components::kill_log())
    //     .track_change(components::kill_log())
    //     .bind(move |v| {
    //         println!("kill log changed: {:?}", v);
    //         run_async(async move {
    //             sleep(5.0).await;
    //             entity::mutate_component(
    //                 entity::synchronized_resources(),
    //                 components::kill_log(),
    //                 |v| {
    //                     if v.len() >= 1 {
    //                         v.remove(0);
    //                     }
    //                 },
    //             );
    //         });
    //     });

    // run_async(async move {
    //     loop {
    //         sleep(20.).await;
    //         if entity::has_component(
    //             entity::synchronized_resources(),
    //             components::kill_log(),
    //         ) {
    //             entity::mutate_component(
    //                 entity::synchronized_resources(),
    //                 components::kill_log(),
    //                 |v| {
    //                     if v.len() >= 1 {
    //                         v.remove(0);
    //                     }
    //                 },
    //             );
    //         }
    //     }
    // });

    query((is_player(), heal_timeout())).each_frame(move |entities| {
        for (e, (_, old_timeout)) in entities {
            let new_timeout = old_timeout - 1;
            entity::set_component(e, heal_timeout(), new_timeout);
        }
    });

    let healables = query((is_player(), health())).build();
    run_async(async move {
        loop {
            sleep(1.0).await;

            for (e, (_, old_health)) in healables.evaluate() {
                let hit_freeze = entity::get_component(e, components::hit_freeze()).unwrap_or(0);
                if hit_freeze > 0 {
                    continue;
                }
                if let Some(timeout) = entity::get_component(e, components::heal_timeout()) {
                    if timeout > 0 {
                        continue;
                    }
                }

                let new_health = old_health + 1.;
                if new_health <= 100. {
                    entity::set_component(e, health(), new_health);
                }
            }
        }
    });
}

fn remove_last_history() {
    run_async(async move {
        sleep(10.0).await;
        entity::mutate_component(
            entity::synchronized_resources(),
            components::kill_log(),
            |v| {
                if !v.is_empty() {
                    v.remove(0);
                }
            },
        );
    });
}
