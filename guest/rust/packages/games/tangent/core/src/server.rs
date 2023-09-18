use std::{f32::consts::PI, sync::OnceLock};

use ambient_api::{
    core::{
        app::components::main_scene,
        ecs::components::remove_at_game_time,
        hierarchy::components::parent,
        messages::Collision,
        model::components::model_from_url,
        physics::components as phyc,
        player::components::is_player,
        primitives::concepts::Sphere,
        rendering::components::cast_shadows,
        transform::components::{
            local_to_parent, local_to_world, mesh_to_local, mesh_to_world, rotation, scale,
            translation,
        },
    },
    ecs::GeneralQuery,
    once_cell::sync::Lazy,
    prelude::*,
};

use packages::{
    tangent_schema::{
        concepts::{Explosion, Spawnpoint, Vehicle},
        explosion,
        messages::OnDeath,
        player::components as pc,
        vehicle::{class::components as vclc, components as vc, data as vd},
    },
    this::messages::{Input, OnCollision, OnSpawn},
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
                .with(scale(), vec3(1.0, 1.0, 1.0 / (2.0 * spawnpoint.radius))),
            );
        }
    });

    // When the player's class changes, respawn them.
    change_query(pc::vehicle_class())
        .track_change(pc::vehicle_class())
        .requires(is_player())
        .bind(move |players| {
            for (player_id, _class_id) in players {
                respawn_player(player_id);
            }
        });

    // When a player despawns (leaves), despawn their vehicle.
    despawn_query(is_player()).bind(|players| {
        for (player, ()) in players {
            if let Some(vehicle) = entity::get_component(player, pc::vehicle_ref()) {
                entity::despawn_recursive(vehicle);
            }
        }
    });

    // When a player sends input, update their input state.
    Input::subscribe(|ctx, input| {
        if let Some(player) = ctx.client_entity_id() {
            entity::add_component(player, pc::input_direction(), input.direction);
            entity::add_component(player, pc::input_jump(), input.jump);

            // If the user opted to commit suicide, immediately destroy their vehicle
            if input.suicide {
                if let Some(vehicle) = entity::get_component(player, pc::vehicle_ref()) {
                    entity::set_component(vehicle, vc::health(), 0.0);
                }
            }
        }
    });

    // When a collision occurs involving a vehicle, damage it.
    Collision::subscribe(|msg| {
        let avg_position = msg
            .ids
            .iter()
            .flat_map(|id| entity::get_component(*id, translation()))
            .reduce(|a, b| (a + b) / 2.0)
            .unwrap_or_default();

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

            OnCollision {
                position: avg_position,
                speed,
                vehicle_id: id,
            }
            .send_client_broadcast_unreliable();
        }
    });

    // Process all vehicles.
    query(vc::player_ref()).each_frame(move |vehicles| {
        for (vehicle_id, driver_id) in vehicles {
            process_vehicle(vehicle_id, driver_id);
        }
    });

    handle_explosions();
}

fn respawn_player(player_id: EntityId) {
    let Some(class_id) = entity::get_component(player_id, pc::vehicle_class()) else {
        return;
    };

    // Copy the class definition, and remove anything class-specific from it.
    let mut class = entity::get_all_components(class_id);
    let Some(model_url) = class.remove(vclc::model_url()) else {
        return;
    };
    let Some(model_scale) = class.remove(vclc::model_scale()) else {
        return;
    };
    class.remove(vclc::is_class());
    class.remove(vclc::name());
    class.remove(vclc::description());
    class.remove(vclc::icon_url());

    let offsets = class
        .get(vd::thruster::components::offsets())
        .unwrap_or_default();

    let last_distances = offsets.iter().map(|_| 0.0).collect();
    let max_health = class
        .get(vd::general::components::max_health())
        .unwrap_or(100.0);

    // Create the vehicle before spawning it.
    let position = choose_spawn_position();
    let vehicle = class
        // Runtime state
        .with(phyc::linear_velocity(), Vec3::ZERO)
        .with(phyc::angular_velocity(), Vec3::ZERO)
        .with(phyc::physics_controlled(), ())
        .with(phyc::dynamic(), true)
        .with(local_to_world(), default())
        .with(translation(), position)
        .with(rotation(), Quat::from_rotation_z(random::<f32>() * PI))
        .with(vc::player_ref(), player_id)
        .with(vc::health(), max_health)
        .with(vc::last_distances(), last_distances)
        .with(vc::last_jump_time(), game_time())
        .with(vc::last_slowdown_time(), game_time());

    assert!(Vehicle::contained_by_unspawned(&vehicle));

    if let Some(existing_vehicle_id) = entity::get_component(player_id, pc::vehicle_ref()) {
        if let Some(translation) = entity::get_component(existing_vehicle_id, translation()) {
            OnDeath {
                position: translation,
                player_id,
            }
            .send_local_broadcast(true);

            Explosion {
                is_explosion: (),
                translation,
                radius: 4.0,
                damage: 25.0,
                optional: default(),
            }
            .spawn();
        }
        entity::despawn_recursive(existing_vehicle_id);
    }

    // Spawn it in.
    let vehicle_id = vehicle.spawn();
    entity::add_component(player_id, pc::vehicle_ref(), vehicle_id);
    entity::add_component(player_id, pc::input_direction(), Vec2::ZERO);
    entity::add_component(player_id, pc::input_jump(), false);

    let _vehicle_model_id = Entity::new()
        .with(cast_shadows(), ())
        .with(model_from_url(), model_url)
        .with(local_to_world(), default())
        .with(local_to_parent(), default())
        .with(mesh_to_local(), default())
        .with(mesh_to_world(), default())
        .with(main_scene(), ())
        .with(scale(), Vec3::ONE * model_scale)
        .with(parent(), vehicle_id)
        .spawn();

    OnSpawn {
        position,
        vehicle_id,
    }
    .send_client_broadcast_unreliable();
}

fn handle_explosions() {
    static QUERY: Lazy<GeneralQuery<Component<Vec3>>> = Lazy::new(|| {
        query(self::translation())
            .requires(vc::player_ref())
            .build()
    });

    spawn_query(Explosion::as_query()).bind(|explosions| {
        for (id, explosion) in explosions {
            let Explosion {
                radius,
                translation,
                damage,
                ..
            } = explosion;

            entity::add_component(
                id,
                remove_at_game_time(),
                game_time() + Duration::from_secs(2),
            );

            physics::add_radial_impulse(
                translation,
                damage * 100.0,
                radius,
                physics::FalloffRadius::FalloffToZeroAt(radius),
            );

            for (vehicle_id, vehicle_translation) in QUERY.evaluate() {
                let distance = vehicle_translation.distance(translation);
                if distance > radius {
                    continue;
                }

                let closeness = (radius - distance) / radius;
                entity::mutate_component(vehicle_id, vc::health(), |health| {
                    *health = (*health - closeness * damage).max(0.0);
                });
            }
        }
    });
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

    sp.translation + (random::<Vec2>() * 2.0 * sp.radius - sp.radius).extend(2.0)
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

    // If the vehicle's been upside down for some time, start applying damage to it.
    if (v.rotation * Vec3::Z).dot(Vec3::Z) < -0.5 {
        if let Some(last_upside_down_time) = v.optional.last_upside_down_time {
            if (game_time() - last_upside_down_time).as_secs_f32() > 0.5 {
                const DAMAGE_PER_SECOND: f32 = 20.0;

                entity::mutate_component(vehicle_id, vc::health(), |health| {
                    *health = (*health - DAMAGE_PER_SECOND * delta_time()).max(0.0);
                });
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
