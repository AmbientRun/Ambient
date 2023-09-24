use ambient_api::{
    core::{
        messages::Collision,
        physics::components as phyc,
        transform::components::{local_to_world, rotation, translation},
    },
    prelude::*,
};

use packages::{
    explosion::concepts::Explosion,
    game_object::components as goc,
    tangent_schema::{
        concepts::{Vehicle, VehicleData},
        vehicle::components as vc,
        weapon,
    },
    this::messages::OnCollision,
};

#[main]
pub fn main() {
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
            .filter(|id| entity::has_component(*id, vc::is_vehicle()))
        {
            let speed = entity::get_component(id, phyc::linear_velocity())
                .map(|v| v.length())
                .unwrap_or_default();

            entity::mutate_component(id, goc::health(), |health| {
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
    query(())
        .requires(vc::is_vehicle())
        .each_frame(move |vehicles| {
            for (vehicle_id, _) in vehicles {
                process_vehicle(vehicle_id);
            }
        });
}

fn process_vehicle(vehicle_id: EntityId) {
    use entity::{get_component as get, set_component as set};
    let Some(v) = Vehicle::get_spawned(vehicle_id) else {
        return;
    };

    // If the vehicle's health is zero, despawn it and spawn an explosion.
    if v.health <= 0.0 {
        Explosion {
            is_explosion: (),
            translation: v.translation,
            radius: 4.0,
            damage: 25.0,
            optional: default(),
        }
        .spawn();

        entity::despawn_recursive(vehicle_id);
        return;
    }

    // If the vehicle's been upside down for some time, start applying damage to it.
    if (v.rotation * Vec3::Z).dot(Vec3::Z) < -0.5 {
        if let Some(last_upside_down_time) = v.optional.last_upside_down_time {
            if (game_time() - last_upside_down_time).as_secs_f32() > 0.5 {
                const DAMAGE_PER_SECOND: f32 = 20.0;

                entity::mutate_component(vehicle_id, goc::health(), |health| {
                    *health = (*health - DAMAGE_PER_SECOND * delta_time()).max(0.0);
                });
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
            entity::add_component(weapon_id, weapon::components::fire(), v.input_fire);
        }
    }

    let Some(vd) = VehicleData::get_spawned(v.data_ref) else {
        return;
    };

    // Apply jump
    let vehicle_last_jump_time = get(vehicle_id, vc::last_jump_time()).unwrap_or_default();
    if v.input_jump && (game_time() - vehicle_last_jump_time) > vd.jump_timeout {
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
            if offset.x * v.input_direction.x < 0.0 {
                vd.turning_strength
            } else {
                0.0
            }
        } else {
            0.0
        };

        let pitch_strength_offset = if thruster_front_of_centre {
            v.input_direction.y * -vd.pitch_strength
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
            * (Vec3::Y * v.input_direction.y.abs())
            * pitch_correction
            * distance_correction
            * -if v.input_direction.y > 0. {
                vd.forward_force
            } else {
                vd.backward_force
            },
        v.translation + v.rotation * vd.forward_offset.extend(0.0),
    );

    // Apply side inputs by applying an invisible force at the front of the vehicle
    physics::add_force_at_position(
        vehicle_id,
        v.rotation * (Vec3::X * v.input_direction.x) * vd.side_force,
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
