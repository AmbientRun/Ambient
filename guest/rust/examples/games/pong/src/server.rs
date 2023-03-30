use std::f32::consts::PI;

use crate::components::player_camera_id;

use ambient_api::{
    components::core::{
        app::{main_scene, window_logical_size},
        camera::*,
        physics::linear_velocity,
        player::{player, user_id},
        primitives::cube,
        rendering,
        transform::*,
    },
    concepts::{make_orthographic_camera, make_sphere, make_transformable},
    player::KeyCode,
    prelude::*,
};

const X_BOUNDARY: f32 = 1.;
const Y_BOUNDARY: f32 = 1.;

const BALL_V_PER_FRAME: f32 = 0.01;
const BALL_ACCELERATION: f32 = 0.05;
const BALL_SPINNING: f32 = PI / 4.; // radians / ratio of paddle from the center (-0.5 - 0.5)
const BALL_RADIUS: f32 = 0.1;
const PADDLE_V_PER_FRAME: f32 = BALL_V_PER_FRAME * 2.;
const PADDLE_LENGTH: f32 = 0.3;
const PADDLE_WIDTH: f32 = 0.1;
const SCREEN_PADDING: f32 = 0.2;

fn spawn_paddle(left: bool, color: Vec3) -> EntityId {
    let x = X_BOUNDARY + PADDLE_WIDTH / 2.;
    make_transformable()
        .with_default(cube())
        .with(scale(), vec3(PADDLE_WIDTH, PADDLE_LENGTH, 1.))
        .with(translation(), vec3(if left { -x } else { x }, 0., 0.))
        .with(rendering::color(), color.extend(1.))
        .spawn()
}

fn gen_ball_velocity() -> Vec3 {
    let angle = random::<f32>() * (PI / 5.) + PI / 10.;
    let y_sign = if random::<bool>() { 1. } else { -1. };
    vec3(
        angle.cos() * BALL_V_PER_FRAME,
        angle.sin() * BALL_V_PER_FRAME * y_sign,
        0.,
    )
}

#[main]
pub fn main() {
    // Spawn field, paddles and ball
    make_transformable()
        .with_default(cube())
        .with(scale(), vec3(X_BOUNDARY * 2., Y_BOUNDARY * 2., 1.))
        .with(translation(), vec3(0., 0., 1.))
        .with(rendering::color(), vec4(1., 1., 1., 1.))
        .spawn();
    let paddles = [
        spawn_paddle(true, vec3(255., 0., 0.)),
        spawn_paddle(false, vec3(0., 255., 0.)),
    ];
    let ball = make_transformable()
        .with_merge(make_sphere())
        .with(scale(), vec3(BALL_RADIUS, BALL_RADIUS, 1.))
        .with(translation(), vec3(0., 0., -1.))
        .with(rendering::color(), vec4(255., 255., 255., 1.))
        .spawn();

    // When a player spawns, create a camera for them
    spawn_query(user_id())
        .requires(player())
        .bind(move |players| {
            for (player, player_user_id) in players {
                let camera_entity_id = make_orthographic_camera()
                    .with_default(main_scene())
                    .with(user_id(), player_user_id)
                    .spawn();
                entity::add_component(player, player_camera_id(), camera_entity_id);
            }
        });

    // Update camera so we have correct aspect ratio
    change_query((player_camera_id(), window_logical_size()))
        .track_change(window_logical_size())
        .bind(move |windows| {
            for (_, (camera_id, window)) in windows {
                let window = window.as_vec2();
                if window.x <= 0. || window.y <= 0. {
                    continue;
                }

                let x_boundary = X_BOUNDARY + SCREEN_PADDING;
                let y_boundary = Y_BOUNDARY + SCREEN_PADDING;
                let (left, right, top, bottom) = if window.x < window.y {
                    (
                        -x_boundary,
                        x_boundary,
                        y_boundary * window.y / window.x,
                        -y_boundary * window.y / window.x,
                    )
                } else {
                    (
                        -x_boundary * window.x / window.y,
                        x_boundary * window.x / window.y,
                        y_boundary,
                        -y_boundary,
                    )
                };
                entity::set_component(camera_id, orthographic_left(), left);
                entity::set_component(camera_id, orthographic_right(), right);
                entity::set_component(camera_id, orthographic_top(), top);
                entity::set_component(camera_id, orthographic_bottom(), bottom);
            }
        });

    // When a player despawns, clean up their objects
    let player_objects_query = query(user_id()).build();
    despawn_query(user_id()).requires(player()).bind({
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
    query((linear_velocity(), translation()))
        .build()
        .each_frame(move |balls| {
            for (id, (velocity, position)) in balls {
                let new_position = position + velocity;
                entity::set_component(id, translation(), new_position);
                if new_position.y.abs() > Y_BOUNDARY - BALL_RADIUS / 2. {
                    // bounce from top and bottom "walls"
                    let new_velocity = vec3(velocity.x, -velocity.y, velocity.z);
                    entity::set_component(id, linear_velocity(), new_velocity);
                }
            }
        });

    on(event::FRAME, move |_| {
        let players = entity::get_all(player());

        // start the ball if we have 2 players and ball has no velocity
        if players.len() >= 2 && entity::get_component(ball, linear_velocity()).is_none() {
            entity::add_component(ball, linear_velocity(), gen_ball_velocity());
        }

        // handle players' input
        for (i, player) in players.into_iter().enumerate() {
            let paddle = paddles[i % 2];
            let Some(input) = player::get_raw_input(player) else { continue; };
            let keys = &input.keys;
            let Some(mut paddle_position) = entity::get_component(paddle, translation()) else { continue; };

            if keys.contains(&KeyCode::Up) {
                paddle_position.y += PADDLE_V_PER_FRAME;
            }
            if keys.contains(&KeyCode::Down) {
                paddle_position.y -= PADDLE_V_PER_FRAME;
            }
            paddle_position.y = paddle_position.y.clamp(
                PADDLE_LENGTH / 2. - Y_BOUNDARY,
                Y_BOUNDARY - PADDLE_LENGTH / 2.,
            );
            entity::set_component(paddle, translation(), paddle_position);
        }

        // paddle bouncing
        if let Some(ball_position) = entity::get_component(ball, translation()) {
            if ball_position.x.abs() > X_BOUNDARY - BALL_RADIUS / 2. {
                let paddle = paddles[(ball_position.x.signum() + 1.) as usize / 2];
                let paddle_position = entity::get_component(paddle, translation()).unwrap();
                if let Some(velocity) = entity::get_component(ball, linear_velocity()) {
                    let new_velocity = if (paddle_position.y - ball_position.y).abs()
                        < PADDLE_LENGTH / 2. + BALL_RADIUS / 2.
                    {
                        // bounce from the paddle

                        // accelerate a bit
                        let new_v_len = (velocity.x.powi(2) + velocity.y.powi(2)).sqrt()
                            * (1. + BALL_ACCELERATION);
                        // adjust the angle to allow for spinning depending on which part of the paddle was hit by the ball
                        let ratio_from_center = (paddle_position.y - ball_position.y)
                            / PADDLE_LENGTH
                            * paddle_position.x.signum();
                        let new_v_angle =
                            velocity.y.atan2(-velocity.x) + BALL_SPINNING * ratio_from_center;
                        vec3(
                            new_v_angle.cos() * new_v_len,
                            new_v_angle.sin() * new_v_len,
                            velocity.z,
                        )
                    } else {
                        // ball passed the paddle

                        // place it back in the center
                        entity::set_component(ball, translation(), Vec3::ZERO);
                        let mut v = gen_ball_velocity();
                        // make it go against the losing player (keep the sign on x)
                        v.x *= velocity.x.signum();
                        v
                    };
                    entity::set_component(ball, linear_velocity(), new_velocity);
                }
            }
        }
    });
}
