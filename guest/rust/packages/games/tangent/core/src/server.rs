use std::sync::OnceLock;

use ambient_api::{
    core::{
        messages::Collision,
        model::components::model_from_url,
        physics::components as phyc,
        player::components::is_player,
        primitives::concepts::Sphere,
        rendering::components::{cast_shadows, color},
        transform::components::{rotation, scale, translation},
    },
    ecs::GeneralQuery,
    prelude::*,
};

use packages::{
    tangent_schema::{
        concepts::{Spawnpoint, Vehicle, VehicleData},
        messages::Input,
        player::components as pc,
        vehicle::components as vc,
    },
    this::assets,
};

#[main]
pub fn main() {
    // When a spawnpoint is created, give it a physical representation.
    spawn_query(Spawnpoint::as_query()).bind(|spawnpoints| {
        for (id, spawnpoint) in spawnpoints {
            entity::add_components(
                id,
                Sphere {
                    sphere: (),
                    sphere_radius: spawnpoint.radius,
                    ..Sphere::suggested()
                }
                .make()
                .with(color(), vec4(1.0, 0.25, 0.0, 1.0))
                .with(scale(), vec3(1.0, 1.0, 1.0 / (2.0 * spawnpoint.radius))),
            );
        }
    });

    // When a player spawns, give them a vehicle.
    spawn_query(is_player()).bind(|players| {
        for (player_id, ()) in players {
            respawn_player(player_id);
        }
    });

    // When a player despawns (leaves), despawn their vehicle.
    despawn_query(is_player()).bind(|players| {
        for (player, ()) in players {
            if let Some(vehicle) = entity::get_component(player, pc::vehicle_ref()) {
                entity::despawn(vehicle);
            }
        }
    });

    // When a player sends input, update their input state.
    Input::subscribe(|ctx, input| {
        if let Some(player) = ctx.client_entity_id() {
            entity::set_component(player, pc::input_direction(), input.direction);
            entity::set_component(player, pc::input_jump(), input.jump);
        }
    });

    // When a collision occurs involving a vehicle, damage it.
    Collision::subscribe(|msg| {
        for id in msg
            .ids
            .iter()
            .copied()
            .filter(|id| entity::has_component(*id, vc::player_ref()))
        {
            let speed = entity::get_component(id, phyc::linear_velocity())
                .map(|v| v.length())
                .unwrap_or_default();

            entity::mutate_component(id, vc::health(), |health| {
                *health = (*health - speed * 0.75).max(0.0);
            });
        }
    });

    // Process all vehicles.
    query(vc::player_ref()).each_frame(move |vehicles| {
        for (vehicle_id, driver_id) in vehicles {
            process_vehicle(vehicle_id, driver_id);
        }
    });
}

fn respawn_player(player_id: EntityId) {
    const X_DISTANCE: f32 = 0.1;
    const Y_DISTANCE: f32 = 0.4;

    let offsets = vec![
        vec2(-X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, -Y_DISTANCE),
        vec2(X_DISTANCE, Y_DISTANCE),
        vec2(-X_DISTANCE, Y_DISTANCE),
    ];
    let last_distances = offsets.iter().map(|_| 0.0).collect();

    let vd = VehicleData {
        density: 10.0,
        max_health: 100.0,
        offsets,
        k_p: 300.0,
        k_d: -600.0,
        target: 2.5,
        max_strength: 25.0,
        forward_force: 50.0,
        backward_force: -4.0,
        forward_offset: vec2(0.0, Y_DISTANCE),
        side_force: 0.8,
        side_offset: vec2(0.0, -Y_DISTANCE),
        jump_force: 80.0,
        pitch_strength: 10.0,
        turning_strength: 20.0,
        jump_timeout: Duration::from_secs_f32(2.0),
        linear_strength: 0.8,
        angular_strength: 0.4,
        angular_delay: Duration::from_secs_f32(0.25),
    };
    let max_health = vd.max_health;

    let vehicle = vd
        .make()
        // Runtime state
        .with(phyc::linear_velocity(), Vec3::ZERO)
        .with(phyc::angular_velocity(), Vec3::ZERO)
        .with(phyc::physics_controlled(), ())
        .with(phyc::dynamic(), true)
        .with(translation(), choose_spawn_position())
        .with(rotation(), Quat::IDENTITY)
        .with(vc::player_ref(), player_id)
        .with(vc::health(), max_health)
        .with(vc::last_distances(), last_distances)
        .with(vc::last_jump_time(), game_time())
        .with(vc::last_slowdown_time(), game_time())
        // Visual appearance
        .with(cast_shadows(), ())
        .with(
            model_from_url(),
            assets::url("models/dynamic/raceCarWhite.glb/models/main.json"),
        )
        .with(phyc::cube_collider(), Vec3::new(0.6, 1.0, 0.2));

    assert!(Vehicle::contained_by_unspawned(&vehicle));

    if let Some(existing_vehicle_id) = entity::get_component(player_id, pc::vehicle_ref()) {
        entity::despawn(existing_vehicle_id);
    }

    let vehicle_id = vehicle.spawn();
    entity::add_component(player_id, pc::vehicle_ref(), vehicle_id);
    entity::add_component(player_id, pc::input_direction(), Vec2::ZERO);
    entity::add_component(player_id, pc::input_jump(), false);
}

fn choose_spawn_position() -> Vec3 {
    static QUERY: OnceLock<GeneralQuery<ConceptQuery<Spawnpoint>>> = OnceLock::new();
    let sp = QUERY
        .get_or_init(|| query(Spawnpoint::as_query()).build())
        .evaluate()
        .choose(&mut thread_rng())
        .map(|(_, sp)| sp)
        .cloned();

    let Some(sp) = sp else {
        return Vec3::ZERO;
    };

    sp.translation + (random::<Vec2>() * 2.0 * sp.radius - sp.radius).extend(5.0)
}

fn process_vehicle(vehicle_id: EntityId, driver_id: EntityId) {
    use entity::{get_component as get, set_component as set};

    let direction = get(driver_id, pc::input_direction()).unwrap_or_default();
    let Some(v) = Vehicle::get_spawned(vehicle_id) else {
        return;
    };

    // If the vehicle's health is at zero, respawn it.
    if v.health <= 0.0 {
        respawn_player(driver_id);
        return;
    }

    // If the vehicle's been upside down for some time, respawn it and don't process any further logic.
    if (v.rotation * Vec3::Z).dot(Vec3::Z) < -0.4 {
        if let Some(last_upside_down_time) = v.optional.last_upside_down_time {
            if (game_time() - last_upside_down_time).as_secs_f32() > 1.5 {
                respawn_player(driver_id);
                return;
            }
        } else {
            entity::add_component(vehicle_id, vc::last_upside_down_time(), game_time());
        }
    } else {
        entity::remove_component(vehicle_id, vc::last_upside_down_time());
    }

    let mut last_distances = v.last_distances;

    // Apply jump
    let vehicle_last_jump_time = get(vehicle_id, vc::last_jump_time()).unwrap_or_default();
    if get(driver_id, pc::input_jump()).unwrap_or_default()
        && (game_time() - vehicle_last_jump_time) > v.jump_timeout
    {
        let linear_velocity = get(vehicle_id, phyc::linear_velocity()).unwrap_or_default();
        let speed_multiplier = (linear_velocity.dot(v.rotation * -Vec3::Y) * 0.3).max(5.0);

        set(vehicle_id, vc::last_jump_time(), game_time());
        physics::add_force(
            vehicle_id,
            v.rotation * Vec3::Z * v.jump_force * speed_multiplier,
        );
    };

    // Apply per-thruster forces
    let mut avg_distance = 0.0;
    for (index, thruster_offset) in v.offsets.iter().enumerate() {
        let offset = thruster_offset.extend(0.0);

        let probe_start = v.translation + v.rotation * (offset - Vec3::Z * 0.1);
        let probe_direction = v.rotation * Vec3::Z * -1.0;

        if probe_direction.z > 0.0 {
            continue;
        }

        let thruster_front_of_centre = offset.y < 0.0;
        let turning_strength_offset = if thruster_front_of_centre {
            if offset.x * direction.x < 0.0 {
                v.turning_strength
            } else {
                0.0
            }
        } else {
            0.0
        };

        let pitch_strength_offset = if thruster_front_of_centre {
            direction.y * -v.pitch_strength
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

            let error_distance = v.target - hit.distance;
            let p = v.k_p * error_distance;
            let d = v.k_d * delta_distance;
            let strength = ((p + d + strength_offset) * delta_time()).clamp(-0.1, v.max_strength);

            let force = -probe_direction * strength;
            let position = v.translation + v.rotation * offset;
            physics::add_force_at_position(vehicle_id, force, position);

            avg_distance = (avg_distance + new_distance) / 2.0;
            last_distances[index] = new_distance;
        }
    }
    set(vehicle_id, vc::last_distances(), last_distances);

    // Apply forward inputs by applying an invisible force at the back of the vehicle
    let pitch = v.rotation.to_euler(glam::EulerRot::YXZ).1;
    let pitch_correction = pitch.cos().powi(3).max(0.0);
    let distance_correction = 1.0 - ((v.target - avg_distance).abs() / v.target).clamp(0.0, 1.0);
    physics::add_force_at_position(
        vehicle_id,
        v.rotation
            * (Vec3::Y * direction.y.abs())
            * pitch_correction
            * distance_correction
            * -if direction.y > 0. {
                v.forward_force
            } else {
                v.backward_force
            },
        v.translation + v.rotation * v.forward_offset.extend(0.0),
    );

    // Apply side inputs by applying an invisible force at the front of the vehicle
    physics::add_force_at_position(
        vehicle_id,
        v.rotation * (Vec3::X * direction.x) * v.side_force,
        v.translation + v.rotation * v.side_offset.extend(0.0),
    );

    // Apply a constant slowdown force
    physics::add_force(
        vehicle_id,
        -get(vehicle_id, phyc::linear_velocity()).unwrap_or_default() * v.linear_strength,
    );

    // Dampen the angular velocity every so often
    let vehicle_last_slowdown_time = get(vehicle_id, vc::last_slowdown_time()).unwrap_or_default();

    if (game_time() - vehicle_last_slowdown_time) > v.angular_delay {
        entity::mutate_component(vehicle_id, phyc::angular_velocity(), |av| {
            *av -= *av * v.angular_strength;
        });
        set(vehicle_id, vc::last_slowdown_time(), game_time());
    }
}
