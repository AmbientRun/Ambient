use std::{f32::consts::PI, sync::OnceLock};

use ambient_api::{
    core::{
        app::components::main_scene,
        hierarchy::components::parent,
        model::components::model_from_url,
        player::components::is_player,
        rendering::components::cast_shadows,
        transform::components::{
            local_to_parent, local_to_world, mesh_to_local, mesh_to_world, scale,
        },
    },
    ecs::GeneralQuery,
    prelude::*,
};

use packages::{
    game_object::{components as goc, player::components as gopc},
    tangent_schema::{
        concepts::{Spawnpoint, Vehicle, VehicleClass, VehicleDef, VehicleOptional},
        player::components as pc,
        vehicle::{components as vc, def as vd},
    },
    this::messages::Input,
};

mod shared;

#[main]
pub fn main() {
    // When the player's class changes, respawn them.
    change_query(pc::vehicle_class())
        .track_change(pc::vehicle_class())
        .requires(is_player())
        .bind(move |players| {
            for (player_id, _class_id) in players {
                spawn_vehicle_for_player(player_id);
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

    // If a player doesn't have a vehicle, but has a class, spawn one for them.
    query(())
        .requires(pc::vehicle_class())
        .excludes(pc::vehicle_ref())
        .each_frame(|players| {
            for (player_id, _) in players {
                spawn_vehicle_for_player(player_id);
            }
        });

    // Sync all vehicle-drivers back to their drivers.
    {
        change_query(vc::driver_ref())
            .track_change(vc::driver_ref())
            .bind(|vehicles| {
                for (vehicle_id, driver_id) in vehicles {
                    entity::add_component(driver_id, pc::vehicle_ref(), vehicle_id);
                    entity::add_component(driver_id, gopc::control_of_entity(), vehicle_id);
                }
            });

        despawn_query(vc::driver_ref()).bind(|vehicles| {
            for (_, driver_id) in vehicles {
                entity::remove_component(driver_id, pc::vehicle_ref());
                entity::remove_component(driver_id, gopc::control_of_entity());
            }
        });
    }

    // When a player sends input, update their input state.
    Input::subscribe(|ctx, input| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };

        entity::add_components(
            player_id,
            Entity::new()
                .with(pc::input_direction(), input.direction)
                .with(pc::input_jump(), input.jump)
                .with(pc::input_fire(), input.fire)
                .with(pc::input_respawn(), input.respawn)
                .with(pc::input_aim_direction(), input.aim_direction),
        );
    });

    // Sync player input state to vehicle input state.
    query((
        pc::input_direction(),
        pc::input_jump(),
        pc::input_fire(),
        pc::input_aim_direction(),
        pc::input_respawn(),
        pc::vehicle_ref(),
    ))
    .each_frame(|players| {
        for (
            _,
            (
                input_direction,
                input_jump,
                input_fire,
                input_aim_direction,
                input_respawn,
                vehicle_id,
            ),
        ) in players
        {
            if !entity::exists(vehicle_id) {
                return;
            }

            // If the user opted to respawn, immediately destroy their vehicle
            if input_respawn {
                entity::set_component(vehicle_id, goc::health(), 0.0);
                return;
            }

            let aim_direction_limits =
                entity::get_component(vehicle_id, vd::input::components::aim_direction_limits())
                    .unwrap_or(Vec2::ONE * 20.0);
            let input_aim_direction =
                input_aim_direction.clamp(-aim_direction_limits, aim_direction_limits);

            // This calculation is a bit circuitous, but it's simpler than breaking out the intermediate
            // calculations
            let p0 = shared::calculate_aim_position(vehicle_id, input_aim_direction, 0.0);
            let p1 = shared::calculate_aim_position(vehicle_id, input_aim_direction, 1.0);

            let hit = physics::raycast(p0, p1 - p0)
                .into_iter()
                .find(|h| h.entity != vehicle_id);

            const RANGE: f32 = 1_000.0;
            // TODO: figure out why not using a fixed long distance breaks the gun-aim calculation
            let aim_position =
                shared::calculate_aim_position(vehicle_id, input_aim_direction, RANGE);
            let aim_distance = hit.map(|h| h.distance).unwrap_or(RANGE);

            entity::add_components(
                vehicle_id,
                Entity::new()
                    .with(vc::input_direction(), input_direction)
                    .with(vc::input_jump(), input_jump)
                    .with(vc::input_fire(), input_fire)
                    .with(vc::input_aim_direction(), input_aim_direction)
                    .with(vc::aim_position(), aim_position)
                    .with(vc::aim_distance(), aim_distance),
            );
        }
    });
}

fn spawn_vehicle_for_player(player_id: EntityId) {
    let Some(class_id) = entity::get_component(player_id, pc::vehicle_class()) else {
        return;
    };

    let Some(class) = VehicleClass::get_spawned(class_id) else {
        return;
    };

    let def_ref = class.def_ref;

    let Some(def) = VehicleDef::get_spawned(def_ref) else {
        return;
    };

    // Spawn the new vehicle.
    let vehicle_id = Vehicle {
        linear_velocity: default(),
        angular_velocity: default(),
        physics_controlled: (),
        dynamic: true,
        density: def.density,
        cube_collider: def.cube_collider,

        local_to_world: default(),
        translation: choose_spawn_position() + Vec3::Z * def.target,
        rotation: Quat::from_rotation_z(random::<f32>() * PI),

        is_vehicle: (),

        health: def.max_health,
        max_health: def.max_health,

        last_distances: def.offsets.iter().map(|_| 0.0).collect(),
        last_jump_time: game_time(),
        last_slowdown_time: game_time(),
        def_ref,

        input_direction: default(),
        input_jump: default(),
        input_fire: default(),
        input_aim_direction: default(),

        optional: VehicleOptional {
            driver_ref: Some(player_id),
            ..default()
        },
    }
    .spawn();

    let _vehicle_model_id = Entity::new()
        .with(cast_shadows(), ())
        .with(model_from_url(), def.model_url)
        .with(local_to_world(), default())
        .with(local_to_parent(), default())
        .with(mesh_to_local(), default())
        .with(mesh_to_world(), default())
        .with(main_scene(), ())
        .with(scale(), Vec3::ONE * def.model_scale)
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

    sp.translation + ((random::<Vec2>() - 0.5) * 2.0 * sp.radius).extend(0.0)
}
