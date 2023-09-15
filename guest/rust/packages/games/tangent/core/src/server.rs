use ambient_api::{
    core::{
        model::components::model_from_url,
        physics::components::{
            angular_velocity, cube_collider, density, dynamic, linear_velocity, physics_controlled,
        },
        player::components::is_player,
        rendering::components::cast_shadows,
        transform::components::{rotation, translation},
    },
    prelude::*,
};

use packages::{
    tangent_schema::{components, messages::Input},
    this::assets,
};

const X_DISTANCE: f32 = 0.1;
const Y_DISTANCE: f32 = 0.4;
const THRUSTERS: [(f32, f32); 4] = [
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

const SPAWN_POSITION: Vec3 = vec3(0., 0., 5.);
const SPAWN_RADIUS: f32 = 20.0;

#[main]
pub fn main() {
    // When a player spawns, give them a vehicle.
    spawn_query(is_player()).bind(|players| {
        for (player_id, ()) in players {
            let vehicle_id = Entity::new()
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
                .with(
                    components::last_distances(),
                    THRUSTERS.map(|_| 0.0).to_vec(),
                )
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

    // When a player despawns (leaves), despawn their vehicle.
    despawn_query(is_player()).bind(|players| {
        for (player, ()) in players {
            if let Some(vehicle) = entity::get_component(player, components::player_vehicle()) {
                entity::despawn(vehicle);
            }
        }
    });

    // When a player sends input, update their input state.
    Input::subscribe(|ctx, input| {
        if let Some(player) = ctx.client_entity_id() {
            entity::set_component(player, components::input_direction(), input.direction);
            entity::set_component(player, components::input_jump(), input.jump);
        }
    });

    // Process all vehicles.
    query(components::vehicle()).each_frame(move |vehicles| {
        for (vehicle_id, driver_id) in vehicles {
            process_vehicle(vehicle_id, driver_id);
        }
    });
}

fn process_vehicle(vehicle_id: EntityId, driver_id: EntityId) {
    use entity::{get_component as get, set_component as set};

    let direction = get(driver_id, components::input_direction()).unwrap_or_default();
    let Some(vehicle_position) = get(vehicle_id, translation()) else {
        return;
    };
    let Some(vehicle_rotation) = get(vehicle_id, rotation()) else {
        return;
    };

    let mut last_distances = get(vehicle_id, components::last_distances()).unwrap();

    // Apply jump
    let vehicle_last_jump_time = get(vehicle_id, components::last_jump_time()).unwrap_or_default();
    if get(driver_id, components::input_jump()).unwrap_or_default()
        && (game_time() - vehicle_last_jump_time).as_secs_f32() > JUMP_TIMEOUT
    {
        let linear_velocity = get(vehicle_id, linear_velocity()).unwrap_or_default();
        let speed_multiplier = (linear_velocity.dot(vehicle_rotation * -Vec3::Y) * 0.3).max(5.0);

        set(vehicle_id, components::last_jump_time(), game_time());
        physics::add_force(
            vehicle_id,
            vehicle_rotation * Vec3::Z * INPUT_JUMP_STRENGTH * speed_multiplier,
        );
    };

    // Apply per-thruster forces
    let mut avg_distance = 0.0;
    for (index, thruster_offset) in THRUSTERS.iter().enumerate() {
        let offset = Vec2::from(*thruster_offset).extend(0.0);

        let probe_start = vehicle_position + vehicle_rotation * (offset - Vec3::Z * 0.1);
        let probe_direction = vehicle_rotation * Vec3::Z * -1.0;

        if probe_direction.z > 0.0 {
            continue;
        }

        let thruster_front_of_centre = offset.y < 0.0;
        let turning_strength_offset = if thruster_front_of_centre {
            if offset.x * direction.x < 0.0 {
                INPUT_TURNING_STRENGTH
            } else {
                0.0
            }
        } else {
            0.0
        };

        let pitch_strength_offset = if thruster_front_of_centre {
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
            let strength = ((p + d + strength_offset) * delta_time()).clamp(-0.1, MAX_STRENGTH);

            let force = -probe_direction * strength;
            let position = vehicle_position + vehicle_rotation * offset;
            physics::add_force_at_position(vehicle_id, force, position);

            avg_distance = (avg_distance + new_distance) / 2.0;
            last_distances[index] = new_distance;
        }
    }
    set(vehicle_id, components::last_distances(), last_distances);

    // Apply forward inputs by applying an invisible force at the back of the vehicle
    let pitch = vehicle_rotation.to_euler(glam::EulerRot::YXZ).1;
    let pitch_correction = pitch.cos().powi(3).max(0.0);
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

    // Apply side inputs by applying an invisible force at the front of the vehicle
    physics::add_force_at_position(
        vehicle_id,
        vehicle_rotation * (Vec3::X * -direction.x) * INPUT_SIDE_FORCE,
        vehicle_position + vehicle_rotation * -Y_DISTANCE * Vec3::Y,
    );

    // If the vehicle's upside down, reset its rotation and teleport it above
    if (vehicle_rotation * Vec3::Z).dot(Vec3::Z) < -0.4 {
        entity::mutate_component(vehicle_id, translation(), |t| *t += Vec3::Z * 7.0);
        set(vehicle_id, rotation(), Quat::IDENTITY);
    }

    // Apply a constant slowdown force
    physics::add_force(
        vehicle_id,
        -get(vehicle_id, linear_velocity()).unwrap_or_default() * SLOWDOWN_STRENGTH,
    );

    // Dampen the angular velocity every so often
    let vehicle_last_slowdown_time =
        get(vehicle_id, components::last_slowdown_time()).unwrap_or_default();

    if (game_time() - vehicle_last_slowdown_time).as_secs_f32() > ANGULAR_SLOWDOWN_DELAY {
        entity::mutate_component(vehicle_id, angular_velocity(), |av| {
            *av -= *av * ANGULAR_SLOWDOWN_STRENGTH;
        });
        set(vehicle_id, components::last_slowdown_time(), game_time());
    }
}
