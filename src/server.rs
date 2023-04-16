use ambient_api::{
    components::core::{
        app::main_scene,
        model::model_from_url,
        physics::{
            angular_velocity, box_collider, density, dynamic, linear_velocity, physics_controlled,
            plane_collider,
        },
        player::player as player_component,
        prefab::prefab_from_url,
        rendering::{cast_shadows, fog_density, light_diffuse, sky, sun, water},
        transform::{rotation, scale, translation},
    },
    concepts::make_transformable,
    prelude::*,
};
use ambient_ui_components::prelude::{fog_color, fog_height_falloff};

mod common;

const X_DISTANCE: f32 = 0.1;
const Y_DISTANCE: f32 = 0.4;
const OFFSETS: [(f32, f32); 4] = [
    (-X_DISTANCE, -Y_DISTANCE),
    (X_DISTANCE, -Y_DISTANCE),
    (X_DISTANCE, Y_DISTANCE),
    (-X_DISTANCE, Y_DISTANCE),
];

const K_P: f32 = 150.0;
const K_D: f32 = -300.0;
const TARGET: f32 = 4.0;
const MAX_STRENGTH: f32 = 10.0;

const INPUT_FORWARD_FORCE: f32 = 40.0;
const INPUT_BACKWARD_FORCE: f32 = -4.0;
const INPUT_SIDE_FORCE: f32 = 0.8;

const INPUT_PITCH_STRENGTH: f32 = 10.0;
const INPUT_TURNING_STRENGTH: f32 = 20.0;
const INPUT_JUMP_STRENGTH: f32 = 800.0;

const DENSITY: f32 = 10.0;
const SLOWDOWN_STRENGTH: f32 = 0.9;

const ANGULAR_SLOWDOWN_DELAY: f32 = 0.25;
const ANGULAR_SLOWDOWN_STRENGTH: f32 = 0.3;

#[main]
pub fn main() {
    make_water();
    make_sun();
    make_track();

    vehicle_creation_and_destruction();
    vehicle_processing();
}

fn make_water() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(water())
        .with_default(physics_controlled())
        .with_default(plane_collider())
        .with(dynamic(), false)
        .with(scale(), Vec3::ONE * 2000.)
        .spawn();
}

fn make_sun() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with_default(rotation())
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_color(), vec3(0.88, 0.37, 0.34))
        .with(fog_density(), 0.01)
        .with(fog_height_falloff(), 0.1)
        .with(rotation(), Quat::from_rotation_y(190.0f32.to_radians()))
        .spawn();
}

fn make_track() {
    Entity::new()
        .with_merge(make_transformable())
        .with(translation(), vec3(-2500., 2500., -300.0))
        .with(scale(), Vec3::ONE * 1.0)
        .with(
            prefab_from_url(),
            asset::url("assets/models/static/map.glb").unwrap(),
        )
        .spawn();
}

fn vehicle_creation_and_destruction() {
    spawn_query(player_component()).bind(|players| {
        for (player_id, ()) in players {
            let vehicle_id = Entity::new()
                .with_merge(make_transformable())
                .with_default(cast_shadows())
                .with_default(linear_velocity())
                .with_default(angular_velocity())
                .with_default(physics_controlled())
                .with(dynamic(), true)
                .with(components::vehicle(), player_id)
                .with(translation(), vec3(0.0, 0.0, 6.0))
                .with(density(), DENSITY)
                .with(components::last_distances(), OFFSETS.map(|_| 0.0).to_vec())
                .with(components::debug_messages(), vec![])
                .with(components::debug_lines(), vec![])
                .with(components::last_jump_time(), 0.0)
                .with(components::last_slowdown_time(), 0.0)
                .with(
                    model_from_url(),
                    asset::url("assets/models/dynamic/raceCarWhite.glb/models/main.json").unwrap(),
                )
                .with(box_collider(), Vec3::new(0.6, 1.0, 0.2))
                .spawn();

            entity::add_component(player_id, components::player_vehicle(), vehicle_id);
            entity::add_component(player_id, components::input_direction(), Vec2::ZERO);
            entity::add_component(player_id, components::input_jump(), false);
            entity::add_component(player_id, components::input_reset(), false);
        }
    });

    despawn_query(player_component()).bind(|players| {
        for (player, ()) in players {
            if let Some(vehicle) = entity::get_component(player, components::player_vehicle()) {
                entity::despawn(vehicle);
            }
        }
    });

    messages::Input::subscribe(|source, input| {
        if let Some(player) = source.client_entity_id() {
            entity::set_component(player, components::input_direction(), input.direction);
            entity::set_component(player, components::input_jump(), input.jump);
            entity::set_component(player, components::input_reset(), input.reset);
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

            if entity::get_component(driver_id, components::input_jump()).unwrap_or_default()
                && time()
                    - entity::get_component(vehicle_id, components::last_jump_time())
                        .unwrap_or_default()
                    > common::JUMP_TIMEOUT
            {
                entity::set_component(vehicle_id, components::last_jump_time(), time());
                physics::add_force(vehicle_id, vehicle_rotation * Vec3::Z * INPUT_JUMP_STRENGTH);
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
                        ((p + d + strength_offset) * frametime()).clamp(-0.1, MAX_STRENGTH);

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

            let distance_correction = 1.0 - ((TARGET - avg_distance).abs() / TARGET).clamp(0.0, 1.0);

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

            if entity::get_component(driver_id, components::input_reset()).unwrap_or_default() {
                entity::set_component(vehicle_id, translation(), Vec3::Z * 7.0);
                entity::set_component(vehicle_id, rotation(), Quat::IDENTITY);
            }

            // Apply a constant slowdown force
            physics::add_force(
                vehicle_id,
                -entity::get_component(vehicle_id, linear_velocity()).unwrap_or_default()
                    * SLOWDOWN_STRENGTH,
            );

            if time()
                - entity::get_component(vehicle_id, components::last_slowdown_time())
                    .unwrap_or_default()
                > ANGULAR_SLOWDOWN_DELAY
            {
                entity::mutate_component(vehicle_id, angular_velocity(), |av| {
                    *av -= *av * ANGULAR_SLOWDOWN_STRENGTH;
                });
                entity::set_component(vehicle_id, components::last_slowdown_time(), time());
            }
        }
    });
}
