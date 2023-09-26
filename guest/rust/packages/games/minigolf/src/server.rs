use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::active_camera,
            concepts::{
                PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
            },
        },
        hierarchy::components::parent,
        messages::Collision,
        model::components::model_from_url,
        physics::components::{
            angular_velocity, collider_from_url, dynamic, kinematic, linear_velocity,
            physics_controlled, sphere_collider,
        },
        player::components::{is_player, user_id},
        prefab::components::prefab_from_url,
        rendering::components::{color, fog_density, light_diffuse, sky, sun, water},
        text::components::{font_size, text},
        transform::{
            components::{
                local_to_parent, local_to_world, mesh_to_local, mesh_to_world, rotation, scale,
                spherical_billboard, translation,
            },
            concepts::{Transformable, TransformableOptional},
        },
    },
    entity::resources,
    prelude::*,
};
use packages::this::{
    assets,
    components::{
        is_ball, next_player_hue, origin, player_ball, player_camera_state, player_color,
        player_indicator, player_indicator_arrow, player_restore_point, player_shoot_requested,
        player_stroke_count, player_text, player_text_container,
    },
    concepts::{PlayerCameraState, PlayerState},
    messages::{Bonk, Hit, Input},
};
use utils::CameraState;

mod utils;

const BALL_RADIUS: f32 = 0.34;

fn create_environment() {
    Entity::new()
        .with(water(), ())
        .with(scale(), Vec3::ONE * 2000.)
        .spawn();

    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-45_f32.to_radians()))
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.)
        .with(main_scene(), ())
        .spawn();

    Entity::new().with(sky(), ()).spawn();

    Entity::new()
        .with(prefab_from_url(), assets::url("level.glb"))
        .with(translation(), Vec3::Z * -0.25)
        .spawn();

    Entity::new()
        .with(model_from_url(), assets::url("fan.glb"))
        .with(collider_from_url(), assets::url("fan.glb"))
        .with(kinematic(), ())
        .with(dynamic(), true)
        .with(angular_velocity(), vec3(0., 90_f32.to_radians(), 0.))
        .with(translation(), vec3(-35., 161., 8.4331))
        .with(rotation(), Quat::from_rotation_z(180_f32.to_radians()))
        .spawn();
}

fn make_golf_ball() -> Entity {
    Transformable {
        local_to_world: Mat4::IDENTITY,
        optional: TransformableOptional {
            translation: Some(Vec3::ZERO),
            rotation: Some(Quat::IDENTITY),
            scale: Some(Vec3::ONE),
        },
    }
    .make()
    .with(is_ball(), ())
    .with(physics_controlled(), ())
    .with(dynamic(), true)
    .with(sphere_collider(), BALL_RADIUS)
    .with(model_from_url(), assets::url("ball.glb"))
}

fn make_text() -> Entity {
    Entity::new()
        .with(
            local_to_parent(),
            Mat4::from_scale(Vec3::ONE * 0.02) * Mat4::from_rotation_x(-180_f32.to_radians()),
        )
        .with(color(), vec4(1., 0., 0., 1.))
        .with(font_size(), 36.)
        .with(main_scene(), ())
        .with(local_to_world(), Default::default())
        .with(mesh_to_local(), Default::default())
        .with(mesh_to_world(), Default::default())
}

#[main]
pub fn main() {
    create_environment();

    // Set the initial next player hue.
    entity::add_component(resources(), next_player_hue(), 0.);

    // When a player spawns, create their player state.
    spawn_query(user_id())
        .requires(is_player())
        .bind(move |players| {
            for (player, player_user_id) in players {
                let next_color = utils::hsv_to_rgb(&[
                    entity::get_component(resources(), next_player_hue()).unwrap_or_default(),
                    0.7,
                    1.0,
                ])
                .extend(1.);
                // 80 + 22.5; pseudo random color, with 16 being unique
                entity::mutate_component(resources(), next_player_hue(), |h| *h += 102.5);

                entity::add_components(
                    player,
                    PlayerState {
                        player_restore_point: vec3(-5f32, 0f32, 20f32),
                        player_stroke_count: 0,
                        player_color: Vec4::ONE,
                    },
                );

                let camera_state = PlayerCameraState::suggested().spawn();
                entity::add_component(player, player_camera_state(), camera_state);

                PerspectiveInfiniteReverseCamera {
                    optional: PerspectiveInfiniteReverseCameraOptional {
                        translation: Some(Vec3::ZERO),
                        rotation: Some(Quat::IDENTITY),
                        main_scene: Some(()),
                        aspect_ratio_from_window: Some(entity::resources()),
                        user_id: Some(player_user_id.clone()),
                        ..default()
                    },
                    ..PerspectiveInfiniteReverseCamera::suggested()
                }
                .make()
                .with(player_camera_state(), camera_state)
                .spawn();

                entity::add_component(player, player_color(), next_color);

                let text = make_text()
                    .with(color(), next_color)
                    .with(user_id(), player_user_id.clone())
                    .with(text(), player_user_id.clone())
                    .with(parent(), player)
                    .spawn();
                entity::add_component(player, player_text(), text);

                entity::add_component(
                    player,
                    player_text_container(),
                    Entity::new()
                        .with(main_scene(), ())
                        .with(local_to_world(), Default::default())
                        .with(spherical_billboard(), ())
                        .with(translation(), vec3(-5., 0., 5.))
                        .spawn(),
                );

                entity::add_component(
                    player,
                    player_ball(),
                    make_golf_ball()
                        .with(color(), next_color)
                        .with(user_id(), player_user_id.clone())
                        .with(translation(), vec3(-5., 0., 10.))
                        .spawn(),
                );

                entity::add_component(
                    player,
                    player_indicator(),
                    Transformable {
                        local_to_world: Mat4::IDENTITY,
                        optional: TransformableOptional {
                            translation: Some(Vec3::ZERO),
                            rotation: Some(Quat::IDENTITY),
                            scale: Some(Vec3::ONE),
                        },
                    }
                    .make()
                    .with(color(), next_color)
                    .with(user_id(), player_user_id.clone())
                    .with(model_from_url(), assets::url("indicator.glb"))
                    .spawn(),
                );

                entity::add_component(
                    player,
                    player_indicator_arrow(),
                    Transformable {
                        local_to_world: Mat4::IDENTITY,
                        optional: TransformableOptional {
                            translation: Some(Vec3::ZERO),
                            rotation: Some(Quat::IDENTITY),
                            scale: Some(Vec3::ONE),
                        },
                    }
                    .make()
                    .with(color(), next_color)
                    .with(user_id(), player_user_id.clone())
                    .with(model_from_url(), assets::url("indicator_arrow.glb"))
                    .spawn(),
                );

                entity::add_component(player, player_shoot_requested(), false);
            }
        });

    let flag = Transformable {
        local_to_world: Mat4::IDENTITY,
        optional: TransformableOptional {
            translation: Some(Vec3::ZERO),
            rotation: Some(Quat::IDENTITY),
            scale: Some(Vec3::ONE),
        },
    }
    .make()
    .with(model_from_url(), assets::url("flag.glb"))
    .with(collider_from_url(), assets::url("flag.glb"))
    .with(dynamic(), true)
    .with(kinematic(), ())
    .with(origin(), vec3(-35., 205., 0.3166))
    .spawn();

    // Update the flag every frame.
    query(translation())
        .requires(is_ball())
        .each_frame(move |balls| {
            let flag_origin = entity::get_component(flag, origin()).unwrap_or_default();
            let mut min_distance = std::f32::MAX;
            for (_, ball_position) in &balls {
                let distance = ball_position.distance(flag_origin);
                if distance < min_distance {
                    min_distance = distance;
                }
            }
            if min_distance < 5. {
                entity::set_component(
                    flag,
                    translation(),
                    flag_origin + Vec3::Z * (5. - min_distance),
                );
            } else {
                entity::set_component(flag, translation(), flag_origin);
            }
        });

    // Update player cameras every frame.
    query(player_camera_state())
        .requires(active_camera())
        .each_frame(move |cameras| {
            for (id, camera_state) in cameras {
                let camera_state = CameraState(camera_state);
                let (camera_translation, camera_rotation) = camera_state.get_transform();
                entity::set_component(id, translation(), camera_translation);
                entity::set_component(id, rotation(), camera_rotation * Quat::from_rotation_x(90.));
            }
        });

    // When a player despawns, clean up their objects.
    let player_objects_query = query(user_id()).build();
    despawn_query(user_id()).requires(is_player()).bind({
        move |players| {
            let player_objects = player_objects_query.evaluate();
            for (_, player_user_id) in &players {
                for (id, _) in player_objects
                    .iter()
                    .filter(|(_, object_user_id)| *player_user_id == *object_user_id)
                {
                    entity::despawn(*id);
                }
            }
        }
    });

    Input::subscribe(|ctx, msg| {
        let Some(user_id) = ctx.client_entity_id() else {
            return;
        };

        if let Some(player_camera_state) = entity::get_component(user_id, player_camera_state()) {
            let player_camera_state = CameraState(player_camera_state);

            player_camera_state.zoom(msg.camera_zoom / 25.);
            if msg.camera_rotation.length_squared() > 0. {
                player_camera_state.rotate(msg.camera_rotation / 250.);
            }

            if msg.shoot {
                entity::set_component(user_id, player_shoot_requested(), true);
            }
        }
    });

    Collision::subscribe(move |msg| {
        // TODO: change msg.ids[0] to the bouncing ball
        Bonk::new(msg.ids[0]).send_client_broadcast_unreliable();
    });

    let start_time = game_time();

    // Update player ball each frame.
    query((
        player_ball(),
        player_text(),
        player_text_container(),
        player_indicator(),
        player_indicator_arrow(),
        player_camera_state(),
        player_shoot_requested(),
    ))
    .requires(is_player())
    .each_frame(move |players| {
        for (
            player,
            (
                player_ball,
                player_text,
                player_text_container,
                player_indicator,
                player_indicator_arrow,
                player_camera_state,
                player_shoot_requested,
            ),
        ) in players
        {
            let player_camera_state = CameraState(player_camera_state);

            let ball_position =
                entity::get_component(player_ball, translation()).unwrap_or_default();

            player_camera_state.set_position(ball_position);

            let can_shoot = {
                let lv = entity::get_component(player_ball, linear_velocity()).unwrap_or_default();
                lv.xy().length_squared() < 1.0 && !is_vertically_moving(lv)
            };

            let force_multiplier = {
                let mut mul = (game_time() - start_time).as_secs_f32() % 2.0;
                if mul > 1.0 {
                    mul = 1.0 - (mul - 1.0);
                }
                mul
            };

            entity::set_component(
                player_text_container,
                translation(),
                ball_position + Vec3::Z * 2.,
            );

            // TODO: This can be removed after #114 is resolved.
            let player_color = entity::get_component(player, player_color()).unwrap_or_default();
            entity::set_component(player_ball, color(), player_color);
            entity::set_component(player_indicator, color(), player_color);
            entity::set_component(player_indicator_arrow, color(), player_color);

            let camera_rotation = Quat::from_rotation_z(player_camera_state.get_yaw());
            let camera_direction = camera_rotation * -Vec3::Y;

            entity::set_component(player_indicator, translation(), ball_position);
            entity::set_component(player_indicator, rotation(), camera_rotation);

            if can_shoot {
                entity::set_component(player_indicator, scale(), vec3(1.0, force_multiplier, 1.0));

                let arrow_position = ball_position + camera_direction * force_multiplier * 10.;
                entity::set_components(
                    player_indicator_arrow,
                    Entity::new()
                        .with(translation(), arrow_position)
                        .with(rotation(), camera_rotation)
                        .with(scale(), Vec3::ONE),
                );
            } else {
                entity::set_component(player_indicator, scale(), Vec3::ZERO);
                entity::set_component(player_indicator_arrow, scale(), Vec3::ZERO);
            }

            if ball_position.z < 0.25 {
                entity::set_component(player_ball, linear_velocity(), Vec3::ZERO);
                entity::set_component(player_ball, angular_velocity(), Vec3::ZERO);
                entity::set_component(
                    player_ball,
                    translation(),
                    entity::get_component(player, player_restore_point()).unwrap_or_default(),
                );
            }

            if player_shoot_requested {
                if can_shoot {
                    entity::set_component(player, player_restore_point(), ball_position);
                    entity::set_component(
                        player_ball,
                        linear_velocity(),
                        camera_direction * 50. * force_multiplier,
                    );
                    Hit::new(player_ball).send_client_broadcast_unreliable();
                    let stroke_count = entity::get_component(player, player_stroke_count())
                        .unwrap_or_default()
                        + 1;
                    entity::set_component(player_text, text(), stroke_count.to_string());
                    entity::set_component(player, player_stroke_count(), stroke_count);
                }
                entity::set_component(player, self::player_shoot_requested(), false);
            }

            // HACK: Artificially slow down ball until https://github.com/AmbientRun/Ambient/issues/182 is available
            physics::add_force(player_ball, {
                let lv = entity::get_component(player_ball, linear_velocity()).unwrap_or_default();
                let lvl = lv.length();
                if lvl > 0.0 && !is_vertically_moving(lv) {
                    -65.0 * delta_time() * lv.xy().extend(0.0) * (1.0 / lvl)
                } else {
                    Vec3::ZERO
                }
            });
        }
    });
}

fn is_vertically_moving(linear_velocity: Vec3) -> bool {
    linear_velocity.z.abs() > 0.1
}
