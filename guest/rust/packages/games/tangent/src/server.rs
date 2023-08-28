use ambient_api::{
    core::{
        app::components::main_scene,
        model::components::model_from_url,
        physics::components::{
            angular_velocity, cube_collider, density, dynamic, linear_velocity, physics_controlled,
            plane_collider,
        },
        player::components::is_player,
        rendering::components::{
            cast_shadows, fog_color, fog_density, fog_height_falloff, light_diffuse, sky, sun,
            water,
        },
        transform::{
            components::{rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};
use packages::this::{assets, components, messages::Input};

const X_DISTANCE: f32 = 0.1;
const Y_DISTANCE: f32 = 0.4;
const OFFSETS: [(f32, f32); 4] = [
    (-X_DISTANCE, -Y_DISTANCE),
    (X_DISTANCE, -Y_DISTANCE),
    (X_DISTANCE, Y_DISTANCE),
    (-X_DISTANCE, Y_DISTANCE),
];

const K_P: f32 = 300.0;
const K_D: f32 = -600.0;
const TARGET: f32 = 2.5;
const MAX_STRENGTH: f32 = 25.0;

const INPUT_FORWARD_FORCE: f32 = 50.0;
const INPUT_BACKWARD_FORCE: f32 = -4.0;
const INPUT_SIDE_FORCE: f32 = 0.8;

const INPUT_PITCH_STRENGTH: f32 = 10.0;
const INPUT_TURNING_STRENGTH: f32 = 20.0;
const INPUT_JUMP_STRENGTH: f32 = 80.0;

const JUMP_TIMEOUT: f32 = 2.0;

const DENSITY: f32 = 10.0;
const SLOWDOWN_STRENGTH: f32 = 0.8;

const ANGULAR_SLOWDOWN_DELAY: f32 = 0.25;
const ANGULAR_SLOWDOWN_STRENGTH: f32 = 0.4;

const SPAWN_POSITION: Vec3 = vec3(800., -670., 5.);
const SPAWN_RADIUS: f32 = 20.0;

#[main]
pub fn main() {
    make_water();
    make_sun();

    vehicle_creation_and_destruction();
    vehicle_processing();
}

fn make_water() {
    Entity::new()
        .with_merge(make_transformable())
        .with(water(), ())
        .with(physics_controlled(), ())
        .with(plane_collider(), ())
        .with(dynamic(), false)
        .with(scale(), Vec3::ONE * 4000.)
        .spawn();
}

fn make_sun() {
    Entity::new()
        .with_merge(make_transformable())
        .with(sky(), ())
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with(sun(), 0.0)
        .with(rotation(), Default::default())
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_color(), vec3(0.88, 0.37, 0.34))
        .with(fog_density(), 0.01)
        .with(fog_height_falloff(), 0.1)
        .with(rotation(), Quat::from_rotation_y(190.0f32.to_radians()))
        .spawn();
}

fn vehicle_creation_and_destruction() {
    spawn_query(is_player()).bind(|players| {
        for (player_id, ()) in players {
            let vehicle_id = Entity::new()
                .with_merge(make_transformable())
                .with(cast_shadows(), ())
                .with(linear_velocity(), Default::default())
                .with(angular_velocity(), Default::default())
                .with(physics_controlled(), ())
                .with(dynamic(), true)
                .with(components::vehicle(), player_id)
                .with(
                    translation(),
                    SPAWN_POSITION + random::<Vec2>().extend(0.0) * SPAWN_RADIUS,
                )
                .with(density(), DENSITY)
                .with(components::last_distances(), OFFSETS.map(|_| 0.0).to_vec())
                .with(components::debug_messages(), vec![])
                .with(components::debug_lines(), vec![])
                .with(components::last_jump_time(), game_time())
                .with(components::last_slowdown_time(), game_time())
                .with(
                    model_from_url(),
                    assets::url("models/dynamic/raceCarWhite.glb/models/main.json"),
                )
                .with(cube_collider(), Vec3::new(0.6, 1.0, 0.2))
                .spawn();

            entity::add_component(player_id, components::player_vehicle(), vehicle_id);
            entity::add_component(player_id, components::input_direction(), Vec2::ZERO);
            entity::add_component(player_id, components::input_jump(), false);
        }
    });

    despawn_query(is_player()).bind(|players| {
        for (player, ()) in players {
            if let Some(vehicle) = entity::get_component(player, components::player_vehicle()) {
                entity::despawn(vehicle);
            }
        }
    });

    Input::subscribe(|source, input| {
        if let Some(player) = source.client_entity_id() {
            entity::set_component(player, components::input_direction(), input.direction);
            entity::set_component(player, components::input_jump(), input.jump);
        }
    });
}

fn vehicle_processing() {
    query(components::vehicle()).each_frame(move |vehicles| {
        for (vehicle_id, driver_id) in vehicles {
            let direction =
                entity::get_component(driver_id, components::input_direction()).unwrap_or_default();

            let vehicle_position = match entity::get_component(vehicle_id, translation()) {
                Some(vehicle_position) => vehicle_position,
                _ => {
                    continue;
                }
            };
            let vehicle_rotation = match entity::get_component(vehicle_id, rotation()) {
                Some(vehicle_rotation) => vehicle_rotation,
                _ => {
                    continue;
                }
            };

            let mut last_distances =
                entity::get_component(vehicle_id, components::last_distances()).unwrap();

            let vehicle_last_jump_time =
                entity::get_component(vehicle_id, components::last_jump_time()).unwrap_or_default();
            if entity::get_component(driver_id, components::input_jump()).unwrap_or_default()
                && (game_time() - vehicle_last_jump_time).as_secs_f32() > JUMP_TIMEOUT
            {
                let linear_velocity =
                    entity::get_component(vehicle_id, linear_velocity()).unwrap_or_default();
                let forward = (linear_velocity.dot(vehicle_rotation * -Vec3::Y) * 0.3).max(5.0);

                entity::set_component(vehicle_id, components::last_jump_time(), game_time());
                physics::add_force(
                    vehicle_id,
                    vehicle_rotation * Vec3::Z * INPUT_JUMP_STRENGTH * forward,
                );
            };

            let mut avg_distance = 0.0;
            for (index, offset) in OFFSETS.iter().enumerate() {
                let offset = Vec2::from(*offset).extend(0.0);

                let probe_start = vehicle_position + vehicle_rotation * (offset - Vec3::Z * 0.1);
                let probe_direction = vehicle_rotation * Vec3::Z * -1.0;

                if probe_direction.z > 0.0 {
                    continue;
                }

                let turning_strength_offset = if offset.y < 0.0 {
                    if offset.x * direction.x < 0.0 {
                        INPUT_TURNING_STRENGTH
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let pitch_strength_offset = if offset.y < 0.0 {
                    direction.y * -INPUT_PITCH_STRENGTH
                } else {
                    0.0
                };
                let strength_offset = turning_strength_offset + pitch_strength_offset;

                if let Some(hit) = physics::raycast(probe_start, probe_direction)
                    .into_iter()
                    .find(|h| h.entity != vehicle_id)
                {
                    let old_distance = last_distances[index];
                    let new_distance = hit.distance;
                    let delta_distance = new_distance - old_distance;

                    let error_distance = TARGET - hit.distance;
                    let p = K_P * error_distance;
                    let d = K_D * delta_distance;
                    let strength =
                        ((p + d + strength_offset) * delta_time()).clamp(-0.1, MAX_STRENGTH);

                    let force = -probe_direction * strength;
                    let position = vehicle_position + vehicle_rotation * offset;
                    physics::add_force_at_position(vehicle_id, force, position);

                    avg_distance = (avg_distance + new_distance) / 2.0;
                    last_distances[index] = new_distance;
                }
            }
            entity::set_component(vehicle_id, components::last_distances(), last_distances);

            let pitch_correction = vehicle_rotation
                .to_euler(glam::EulerRot::YXZ)
                .1
                .cos()
                .powi(3)
                .max(0.0);

            let distance_correction =
                1.0 - ((TARGET - avg_distance).abs() / TARGET).clamp(0.0, 1.0);

            physics::add_force_at_position(
                vehicle_id,
                vehicle_rotation
                    * (Vec3::Y * direction.y.abs())
                    * pitch_correction
                    * distance_correction
                    * -if direction.y > 0. {
                        INPUT_FORWARD_FORCE
                    } else {
                        INPUT_BACKWARD_FORCE
                    },
                vehicle_position + vehicle_rotation * Y_DISTANCE * Vec3::Y,
            );

            physics::add_force_at_position(
                vehicle_id,
                vehicle_rotation * (Vec3::X * -direction.x) * INPUT_SIDE_FORCE,
                vehicle_position + vehicle_rotation * -Y_DISTANCE * Vec3::Y,
            );

            if (vehicle_rotation * Vec3::Z).dot(Vec3::Z) < -0.4 {
                entity::mutate_component(vehicle_id, translation(), |t| *t += Vec3::Z * 7.0);
                entity::set_component(vehicle_id, rotation(), Quat::IDENTITY);
            }

            // Apply a constant slowdown force
            physics::add_force(
                vehicle_id,
                -entity::get_component(vehicle_id, linear_velocity()).unwrap_or_default()
                    * SLOWDOWN_STRENGTH,
            );

            let vehicle_last_slowdown_time =
                entity::get_component(vehicle_id, components::last_slowdown_time())
                    .unwrap_or_default();
            if (game_time() - vehicle_last_slowdown_time).as_secs_f32() > ANGULAR_SLOWDOWN_DELAY {
                entity::mutate_component(vehicle_id, angular_velocity(), |av| {
                    *av -= *av * ANGULAR_SLOWDOWN_STRENGTH;
                });
                entity::set_component(vehicle_id, components::last_slowdown_time(), game_time());
            }
        }
    });
}
