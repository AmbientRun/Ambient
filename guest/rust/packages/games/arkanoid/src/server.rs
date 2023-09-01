use std::f32::consts::PI;
use std::vec::Vec;

use ambient_api::{
    core::{
        messages::Frame,
        physics::components::linear_velocity,
        player::components::{is_player, user_id},
        primitives::{components::cube, concepts::make_sphere},
        rendering::components::color,
        transform::{components::*, concepts::make_transformable},
    },
    prelude::*,
};

mod constants;
use constants::*;
use packages::this::{
    assets,
    components::{player_movement_direction, track_audio_url},
    messages::{Input, Ping},
};

fn spawn_enemies(enemies: &mut Vec<EntityId>, y_pos: f32, color: Vec3) {
    for i in 0..7 {
        enemies.push(
            make_transformable()
                .with(cube(), ())
                .with(scale(), vec3(PADDLE_WIDTH, PADDLE_HEIGHT / 2., 1.))
                .with(translation(), vec3(-1. + (i as f32 / 3.), y_pos, 0.))
                .with(self::color(), color.extend(1.))
                .spawn(),
        );
    }
}

fn gen_ball_velocity() -> Vec3 {
    let angle = random::<f32>() * (PI / 5.) + PI / 10.;
    let x_sign = if random::<bool>() { 1. } else { -1. };
    vec3(
        angle.cos() * BALL_V_PER_FRAME * x_sign,
        angle.sin() * BALL_V_PER_FRAME,
        0.,
    )
}

#[main]
pub fn main() {
    let bounce_url = assets::url("paddle_bounce.wav");

    entity::add_component(
        entity::synchronized_resources(),
        track_audio_url(),
        bounce_url,
    );

    let mut enemies: Vec<EntityId> = Vec::new();

    spawn_enemies(&mut enemies, 1., vec3(0.86, 0.86, 0.86));
    spawn_enemies(&mut enemies, 0.9, vec3(1., 0., 0.));
    spawn_enemies(&mut enemies, 0.8, vec3(0., 0., 1.));
    spawn_enemies(&mut enemies, 0.7, vec3(1., 0.65, 0.));
    spawn_enemies(&mut enemies, 0.6, vec3(1., 0.71, 0.75));
    spawn_enemies(&mut enemies, 0.5, vec3(0.6, 0.8, 0.2));

    //Spawn field
    make_transformable()
        .with(cube(), ())
        .with(scale(), vec3(X_BOUNDARY * 2.5, Y_BOUNDARY * 2.3, 1.))
        .with(translation(), vec3(0., 0., 1.0))
        .with(self::color(), vec4(1., 1., 1., 1.))
        .spawn();

    make_transformable()
        .with(cube(), ())
        .with(
            scale(),
            vec3(X_BOUNDARY * 2.5 - 0.1, Y_BOUNDARY * 2.3 - 0.1, 1.),
        )
        .with(translation(), vec3(0., 0., 0.9))
        .with(self::color(), vec4(0., 0., 0., 1.))
        .spawn();

    let paddle = make_transformable()
        .with(cube(), ())
        .with(scale(), vec3(PADDLE_WIDTH, PADDLE_HEIGHT, 1.))
        .with(translation(), vec3(0., -0.9, 0.))
        .with(self::color(), vec4(0., 1., 1., 1.))
        .spawn();

    let ball = make_transformable()
        .with_merge(make_sphere())
        .with(scale(), vec3(BALL_RADIUS, BALL_RADIUS, 1.))
        .with(translation(), vec3(0., -0.9 + BALL_RADIUS, 0.))
        .with(self::color(), vec4(1., 1., 1., 1.))
        .spawn();

    // When a player spawns, create a camera and other components for them
    spawn_query(is_player()).bind(move |players| {
        for (player, _) in players {
            entity::add_component(player, player_movement_direction(), 0.0);
        }
    });

    // When a player despawns, clean up their objects
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

    // Ball movement
    query((linear_velocity(), translation())).each_frame(move |balls| {
        for (id, (velocity, position)) in balls {
            let new_position = position + velocity;
            entity::set_component(id, translation(), new_position);

            if new_position.x.abs() > X_BOUNDARY + 0.2 - BALL_RADIUS / 2. {
                // bounce from side "walls"
                let new_velocity = vec3(-velocity.x, velocity.y, velocity.z);
                entity::set_component(id, linear_velocity(), new_velocity);
            }

            if new_position.y > Y_BOUNDARY + 0.1 - BALL_RADIUS / 2. {
                // bounce from top "wall"
                let new_velocity = vec3(velocity.x, -velocity.y, velocity.z);
                entity::set_component(id, linear_velocity(), new_velocity);
            }
        }
    });

    Input::subscribe(move |ctx, msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };

        entity::set_component(player_id, player_movement_direction(), msg.direction);

        if msg.start {
            // start the ball if it has no velocity
            if entity::get_component(ball, linear_velocity()).is_none() {
                entity::add_component(ball, linear_velocity(), gen_ball_velocity());
            }
        }
    });

    Frame::subscribe(move |_| {
        let players = entity::get_all(is_player());

        // handle players' input
        for (_i, player) in players.into_iter().enumerate() {
            let Some(direction) = entity::get_component(player, player_movement_direction()) else {
                continue;
            };
            let Some(mut paddle_position) = entity::get_component(paddle, translation()) else {
                continue;
            };

            paddle_position.x += direction * PADDLE_V_PER_FRAME;
            paddle_position.x = paddle_position
                .x
                .clamp(-0.05 - X_BOUNDARY, X_BOUNDARY + 0.05);
            entity::set_component(paddle, translation(), paddle_position);

            if entity::get_component(ball, linear_velocity()).is_none() {
                // move the ball with the paddle
                let Some(mut ball_position) = entity::get_component(ball, translation()) else {
                    continue;
                };
                ball_position.x = paddle_position.x;
                entity::set_component(ball, translation(), ball_position);
            }
        }

        if let Some(mut ball_position) = entity::get_component(ball, translation()) {
            let paddle_position = entity::get_component(paddle, translation()).unwrap();
            if let Some(velocity) = entity::get_component(ball, linear_velocity()) {
                if (paddle_position.y - ball_position.y).abs()
                    < PADDLE_HEIGHT / 2. + BALL_RADIUS / 2.
                {
                    if ball_position.x > paddle_position.x - PADDLE_WIDTH / 2.
                        && ball_position.x < paddle_position.x + PADDLE_WIDTH / 2.
                    {
                        // bounce from the paddle
                        Ping::new().send_client_broadcast_reliable();
                        // accelerate a bit
                        let new_v_len = (velocity.x.powi(2) + velocity.y.powi(2)).sqrt()
                            * (1. + BALL_ACCELERATION);
                        // adjust the angle to allow for spinning depending on which part of the paddle was hit by the ball
                        let ratio_from_center = (paddle_position.x - ball_position.x)
                            / PADDLE_WIDTH
                            * paddle_position.y.signum();
                        let new_v_angle =
                            velocity.x.atan2(-velocity.y) + BALL_SPINNING * ratio_from_center;
                        let new_velocity = vec3(
                            new_v_angle.cos() * new_v_len,
                            new_v_angle.sin() * new_v_len,
                            velocity.z,
                        );
                        entity::set_component(ball, linear_velocity(), new_velocity);
                    } else {
                        // ball passed the paddle
                        if ball_position.y < -Y_BOUNDARY + BALL_RADIUS {
                            // place it back in the paddle
                            ball_position.x = paddle_position.x;
                            ball_position.y = paddle_position.y + BALL_RADIUS;
                            entity::set_component(ball, translation(), ball_position);
                            entity::remove_component(ball, linear_velocity());
                        }
                    };
                }
            }
        }

        // handle ball hitting the enemy
        if let Some(ball_position) = entity::get_component(ball, translation()) {
            for enemy_id in &enemies {
                if let Some(enemy_position) = entity::get_component(*enemy_id, translation()) {
                    if (enemy_position.y - ball_position.y).abs()
                        < PADDLE_HEIGHT / 2. + BALL_RADIUS / 2.
                        && ball_position.x > enemy_position.x - PADDLE_WIDTH / 2.
                        && ball_position.x < enemy_position.x + PADDLE_WIDTH / 2.
                    {
                        entity::despawn(*enemy_id);

                        // bounce from the enemy
                        if let Some(velocity) = entity::get_component(ball, linear_velocity()) {
                            // adjust the angle to allow for spinning depending on which part of the paddle was hit by the ball
                            // accelerate a bit
                            let new_v_len = (velocity.x.powi(2) + velocity.y.powi(2)).sqrt()
                                * (1. + BALL_ACCELERATION);
                            let ratio_from_center = (enemy_position.x - ball_position.x)
                                / PADDLE_WIDTH
                                * enemy_position.y.signum();
                            let new_v_angle =
                                velocity.x.atan2(-velocity.y) + BALL_SPINNING * ratio_from_center;
                            let new_velocity = vec3(
                                new_v_angle.cos() * new_v_len,
                                new_v_angle.sin() * new_v_len,
                                velocity.z,
                            );
                            entity::set_component(ball, linear_velocity(), new_velocity);

                            break;
                        }
                    }
                }
            }
        }
    });
}
