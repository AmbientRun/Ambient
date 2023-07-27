// TODO: this should vary based on the game type

use ambient_api::components::core::{
    player::player,
    // primitives::cube,
    // rendering::color,
    transform::translation, // rotation, scale,
};
// use ambient_api::concepts::make_transformable;
use ambient_api::prelude::*;
use components::{heal_timeout, player_health};

use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        physics::{
            angular_velocity, cube_collider, dynamic, linear_velocity, physics_controlled,
            plane_collider, sphere_collider,
        },
        primitives::{cube, quad, sphere_radius},
        rendering::{cast_shadows, color, fog_density, light_diffuse, sky, sun, water},
        transform::{lookat_target, rotation, scale},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};

#[main]
pub fn main() {
    spawn_query(player()).bind(|results| {
        for (id, ()) in results {
            run_async(async move {
                entity::wait_for_component(id, components::player_name()).await;
                entity::add_component(id, components::player_health(), 100);
                entity::add_component(id, components::hit_freeze(), 0);
                entity::add_component(id, components::player_killcount(), 0);
                entity::add_component(id, components::player_deathcount(), 0);
                entity::add_component(id, components::heal_timeout(), 0);
            });
        }
    });

    messages::Shoot::subscribe(move |_source, msg| {
        let result = physics::raycast_first(msg.ray_origin, msg.ray_dir);

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
            //         .with_default(cube())
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

            if entity::has_component(hit.entity, components::player_team()) {
                let pos = entity::get_component(hit.entity, translation()).unwrap();
                messages::Explosion { pos }.send_local_broadcast(false);
                let c = entity::get_component(hit.entity, color()).unwrap();
                entity::despawn(hit.entity);
                run_async(async move {
                    for _ in 0..40 {
                        let pos =
                            pos + vec3(random::<f32>(), random::<f32>(), random::<f32>()) * 9.0;

                        let size = random::<Vec3>() * 0.3;
                        let rot = Quat::from_rotation_y(random::<f32>() * 3.14)
                            * Quat::from_rotation_x(random::<f32>() * 3.14);
                        let id = Entity::new()
                            .with_merge(make_transformable())
                            .with_default(cube())
                            .with(rotation(), rot)
                            .with_default(physics_controlled())
                            .with_default(cast_shadows())
                            .with(
                                linear_velocity(),
                                vec3(
                                    random::<f32>() * 3.0,
                                    random::<f32>() * 5.0,
                                    random::<f32>() * 10.0,
                                ),
                            )
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

            if let Some(old_health) = entity::get_component(hit.entity, components::player_health())
            {
                if old_health <= 0 {
                    return;
                }

                let new_health = (old_health - 10).max(0);
                entity::set_component(hit.entity, components::player_health(), new_health);

                if old_health > 0 && new_health <= 0 {
                    println!("player dead, waiting for respawn");
                    // 114 is the death anim frame count
                    entity::set_component(hit.entity, components::hit_freeze(), 114);
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
                    // TODO: wait for anim msg to respawn
                    run_async(async move {
                        sleep(114. / 60.).await;

                        if !entity::exists(hit.entity) {
                            return;
                        }

                        entity::set_component(
                            hit.entity,
                            translation(),
                            vec3(random::<f32>() * 10.0, random::<f32>() * 10.0, 2.0),
                        );
                        entity::set_component(hit.entity, components::player_health(), 100);
                        entity::set_component(hit.entity, components::hit_freeze(), 0);
                    });
                } else {
                    entity::set_component(hit.entity, components::hit_freeze(), 20);
                    entity::set_component(hit.entity, heal_timeout(), 150);
                }
            }
        }
    });

    query((player(), heal_timeout())).each_frame(move |entities| {
        for (e, (_, old_timeout)) in entities {
            let new_timeout = old_timeout - 1;
            entity::set_component(e, heal_timeout(), new_timeout);
        }
    });

    let healables = query((player(), player_health())).build();
    run_async(async move {
        loop {
            sleep(1.0).await;

            for (e, (_, old_health)) in healables.evaluate() {
                if let Some(timeout) = entity::get_component(e, components::heal_timeout()) {
                    if timeout > 0 {
                        continue;
                    }
                }

                let new_health = old_health + 1;
                if new_health <= 100 {
                    entity::set_component(e, components::player_health(), new_health);
                }
            }
        }
    });
}
