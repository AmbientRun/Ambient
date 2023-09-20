use std::{f32::consts::PI, sync::OnceLock};

use ambient_api::{
    core::{
        app::components::main_scene,
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
    prelude::*,
};

use packages::{
    explosion::concepts::Explosion,
    game_object::components::{health, max_health},
    tangent_schema::{
        concepts::{Spawnpoint, Vehicle, VehicleClass, VehicleData},
        messages::OnDeath,
        player::components as pc,
        vehicle::{components as vc, data as vd},
    },
    this::messages::{Input, OnCollision},
};

use crate::packages::tangent_schema::weapon;

mod shared;

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
        let Some(player) = ctx.client_entity_id() else {
            return;
        };

        let Some(vehicle) = entity::get_component(player, pc::vehicle_ref()) else {
            return;
        };

        let aim_direction_limits =
            entity::get_component(vehicle, vd::input::components::aim_direction_limits())
                .unwrap_or(Vec2::ONE * 20.0);

        entity::add_component(player, pc::input_direction(), input.direction);
        entity::add_component(player, pc::input_jump(), input.jump);
        entity::add_component(player, pc::input_fire(), input.fire);
        let aim_direction = input
            .aim_direction
            .clamp(-aim_direction_limits, aim_direction_limits);
        entity::add_component(player, pc::input_aim_direction(), aim_direction);

        entity::add_component(
            vehicle,
            vc::aim_position(),
            shared::calculate_aim_position(vehicle, aim_direction),
        );

        // If the user opted to respawn, immediately destroy their vehicle
        if input.respawn {
            entity::set_component(vehicle, health(), 0.0);
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

            entity::mutate_component(id, health(), |health| {
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
}

fn respawn_player(player_id: EntityId) {
    let Some(class_id) = entity::get_component(player_id, pc::vehicle_class()) else {
        return;
    };

    let Some(class) = VehicleClass::get_spawned(class_id) else {
        return;
    };

    let vehicle_data_ref = class.data_ref;

    let offsets = entity::get_component(vehicle_data_ref, vd::thruster::components::offsets())
        .unwrap_or_default();

    let last_distances = offsets.iter().map(|_| 0.0).collect();
    let max_health = entity::get_component(vehicle_data_ref, max_health()).unwrap_or(100.0);

    // Create the vehicle before spawning it.
    let position = choose_spawn_position();
    let vehicle = Vehicle {
        linear_velocity: default(),
        angular_velocity: default(),
        physics_controlled: (),
        dynamic: true,
        density: entity::get_component(vehicle_data_ref, phyc::density()).unwrap_or_default(),
        cube_collider: entity::get_component(vehicle_data_ref, phyc::cube_collider())
            .unwrap_or_default(),

        local_to_world: default(),
        translation: position,
        rotation: Quat::from_rotation_z(random::<f32>() * PI),
        player_ref: player_id,
        health: max_health,
        last_distances,
        last_jump_time: game_time(),
        last_slowdown_time: game_time(),
        data_ref: vehicle_data_ref,
        optional: default(),
    }
    .make();

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
    entity::add_component(player_id, pc::input_fire(), false);
    entity::add_component(player_id, pc::input_aim_direction(), Vec2::ZERO);

    let _vehicle_model_id = Entity::new()
        .with(cast_shadows(), ())
        .with(model_from_url(), class.model_url)
        .with(local_to_world(), default())
        .with(local_to_parent(), default())
        .with(mesh_to_local(), default())
        .with(mesh_to_world(), default())
        .with(main_scene(), ())
        .with(scale(), Vec3::ONE * class.model_scale)
        .with(parent(), vehicle_id)
        .spawn();
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
    let fire = get(driver_id, pc::input_fire()).unwrap_or_default();

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

                entity::mutate_component(vehicle_id, health(), |health| {
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

    // Process gun aiming and shooting
    if let Some((aimable_weapon_refs, aim_position)) =
        v.optional.aimable_weapon_refs.zip(v.optional.aim_position)
    {
        for weapon_id in aimable_weapon_refs {
            let Some(weapon_ltw) = entity::get_component(weapon_id, local_to_world()) else {
                continue;
            };

            let (_, _, weapon_position) = weapon_ltw.to_scale_rotation_translation();

            let inv_local_to_world =
                Mat4::from_rotation_translation(-v.rotation, -weapon_position).transpose();

            let aim_position_relative_to_gun = inv_local_to_world.transform_point3(aim_position);
            let rot = Quat::from_rotation_arc(-Vec3::Y, aim_position_relative_to_gun.normalize());

            entity::set_component(weapon_id, rotation(), rot);
            entity::add_component(weapon_id, weapon::components::fire(), fire);
        }
    }

    let Some(vd) = VehicleData::get_spawned(v.data_ref) else {
        return;
    };

    // Apply jump
    let vehicle_last_jump_time = get(vehicle_id, vc::last_jump_time()).unwrap_or_default();
    if get(driver_id, pc::input_jump()).unwrap_or_default()
        && (game_time() - vehicle_last_jump_time) > vd.jump_timeout
    {
        let linear_velocity = get(vehicle_id, phyc::linear_velocity()).unwrap_or_default();
        let speed_multiplier = (linear_velocity.dot(v.rotation * -Vec3::Y) * 0.3).max(5.0);

        set(vehicle_id, vc::last_jump_time(), game_time());
        physics::add_force(
            vehicle_id,
            v.rotation * Vec3::Z * vd.jump_force * speed_multiplier,
        );
    };

    // Apply per-thruster forces
    let mut last_distances = v.last_distances;
    let mut avg_distance = 0.0;
    for (index, thruster_offset) in vd.offsets.iter().enumerate() {
        let offset = thruster_offset.extend(0.0);

        let probe_start = v.translation + v.rotation * (offset - Vec3::Z * 0.1);
        let probe_direction = v.rotation * Vec3::Z * -1.0;

        if probe_direction.z > 0.0 {
            continue;
        }

        let thruster_front_of_centre = offset.y < 0.0;
        let turning_strength_offset = if thruster_front_of_centre {
            if offset.x * direction.x < 0.0 {
                vd.turning_strength
            } else {
                0.0
            }
        } else {
            0.0
        };

        let pitch_strength_offset = if thruster_front_of_centre {
            direction.y * -vd.pitch_strength
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

            let error_distance = vd.target - hit.distance;
            let p = vd.k_p * error_distance;
            let d = vd.k_d * delta_distance;
            let strength = ((p + d + strength_offset) * delta_time()).clamp(-0.1, vd.max_strength);

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
    let distance_correction = 1.0 - ((vd.target - avg_distance).abs() / vd.target).clamp(0.0, 1.0);
    physics::add_force_at_position(
        vehicle_id,
        v.rotation
            * (Vec3::Y * direction.y.abs())
            * pitch_correction
            * distance_correction
            * -if direction.y > 0. {
                vd.forward_force
            } else {
                vd.backward_force
            },
        v.translation + v.rotation * vd.forward_offset.extend(0.0),
    );

    // Apply side inputs by applying an invisible force at the front of the vehicle
    physics::add_force_at_position(
        vehicle_id,
        v.rotation * (Vec3::X * direction.x) * vd.side_force,
        v.translation + v.rotation * vd.side_offset.extend(0.0),
    );

    // Apply a constant slowdown force
    physics::add_force(
        vehicle_id,
        -get(vehicle_id, phyc::linear_velocity()).unwrap_or_default() * vd.linear_strength,
    );

    // Dampen the angular velocity every so often
    let vehicle_last_slowdown_time = get(vehicle_id, vc::last_slowdown_time()).unwrap_or_default();

    if (game_time() - vehicle_last_slowdown_time) > vd.angular_delay {
        entity::mutate_component(vehicle_id, phyc::angular_velocity(), |av| {
            *av -= *av * vd.angular_strength;
        });
        set(vehicle_id, vc::last_slowdown_time(), game_time());
    }
}
