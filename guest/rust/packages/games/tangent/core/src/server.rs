use std::{f32::consts::PI, sync::OnceLock};

use ambient_api::{
    core::{
        app::components::main_scene,
        hierarchy::components::parent,
        model::components::model_from_url,
        physics::components as phyc,
        player::components::is_player,
        rendering::components::cast_shadows,
        transform::components::{
            local_to_parent, local_to_world, mesh_to_local, mesh_to_world, scale, translation,
        },
    },
    ecs::GeneralQuery,
    prelude::*,
};

use packages::{
    explosion::concepts::Explosion,
    game_object::{components as goc, player::components as gopc},
    tangent_schema::{
        concepts::{Spawnpoint, Vehicle, VehicleClass, VehicleOptional},
        messages::OnDeath,
        player::components as pc,
        vehicle::{components as vc, data as vd},
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

    // When a player('s vehicle) dies, respawn them.
    // TODO: decouple
    query((vc::driver_ref(), goc::health())).each_frame(|vehicles| {
        for (_, (player_id, health)) in vehicles {
            if health <= 0.0 {
                respawn_player(player_id);
            }
        }
    });

    // When a player sends input, update their input state.
    Input::subscribe(|ctx, input| {
        let Some(player) = ctx.client_entity_id() else {
            return;
        };

        let Some(vehicle_id) = entity::get_component(player, pc::vehicle_ref()) else {
            return;
        };

        let aim_direction_limits =
            entity::get_component(vehicle_id, vd::input::components::aim_direction_limits())
                .unwrap_or(Vec2::ONE * 20.0);

        entity::add_component(player, pc::input_direction(), input.direction);
        entity::add_component(player, pc::input_jump(), input.jump);
        entity::add_component(player, pc::input_fire(), input.fire);
        let aim_direction = input
            .aim_direction
            .clamp(-aim_direction_limits, aim_direction_limits);
        entity::add_component(player, pc::input_aim_direction(), aim_direction);

        // This calculation is a bit circuitous, but it's simpler than breaking out the intermediate
        // calculations
        let p0 = shared::calculate_aim_position(vehicle_id, aim_direction, 0.0);
        let p1 = shared::calculate_aim_position(vehicle_id, aim_direction, 1.0);

        let hit = physics::raycast(p0, p1 - p0)
            .into_iter()
            .find(|h| h.entity != vehicle_id);

        const RANGE: f32 = 1_000.0;
        // TODO: figure out why not using a fixed long distance breaks the gun-aim calculation
        let aim_position = shared::calculate_aim_position(vehicle_id, aim_direction, RANGE);
        let aim_distance = hit.map(|h| h.distance).unwrap_or(RANGE);

        entity::add_component(vehicle_id, vc::aim_position(), aim_position);
        entity::add_component(vehicle_id, vc::aim_distance(), aim_distance);

        // If the user opted to respawn, immediately destroy their vehicle
        if input.respawn {
            entity::set_component(vehicle_id, goc::health(), 0.0);
        }
    });

    // Sync player input state to vehicle input state.
    query((
        pc::input_direction(),
        pc::input_jump(),
        pc::input_fire(),
        pc::input_aim_direction(),
        pc::vehicle_ref(),
    ))
    .each_frame(|players| {
        for (_, (input_direction, input_jump, input_fire, input_aim_direction, vehicle_ref)) in
            players
        {
            entity::add_components(
                vehicle_ref,
                Entity::new()
                    .with(vc::input_direction(), input_direction)
                    .with(vc::input_jump(), input_jump)
                    .with(vc::input_fire(), input_fire)
                    .with(vc::input_aim_direction(), input_aim_direction),
            );
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
    let max_health = entity::get_component(vehicle_data_ref, goc::max_health()).unwrap_or(100.0);

    // Kill the existing vehicle if it exists.
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

    // Spawn the new vehicle.
    let position = choose_spawn_position();
    let vehicle_id = Vehicle {
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

        is_vehicle: (),

        health: max_health,
        max_health,

        last_distances,
        last_jump_time: game_time(),
        last_slowdown_time: game_time(),
        data_ref: vehicle_data_ref,

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
    entity::add_component(player_id, pc::vehicle_ref(), vehicle_id);
    entity::add_component(player_id, gopc::control_of_entity(), vehicle_id);

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
