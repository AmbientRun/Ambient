use std::{f32::consts::PI, sync::OnceLock};

use ambient_api::{
    core::{
        app::components::name,
        player::components::{is_player, user_id},
        transform::components::{rotation, translation},
    },
    ecs::GeneralQuery,
    prelude::*,
};

use packages::{
    game_object::{components as goc, player::components as gopc},
    tangent_schema::{
        concepts::Spawnpoint,
        player::character::components as pcc,
        player::components as pc,
        vehicle::{components as vc, def as vd},
    },
    this::messages::Input,
    unit_schema::components as uc,
};

mod shared;

#[main]
pub fn main() {
    // When the player's class changes, respawn them.
    change_query(pc::class())
        .track_change(pc::class())
        .requires(is_player())
        .bind(move |players| {
            for (player_id, _class_id) in players {
                if let Some(character_ref) = entity::get_component(player_id, pc::character_ref()) {
                    entity::despawn_recursive(character_ref);
                }

                let character_id = Entity::new()
                    .with(pcc::is_character(), ())
                    .with(pcc::player_ref(), player_id)
                    .with(translation(), choose_spawn_position())
                    .with(
                        name(),
                        format!(
                            "{}'s Character",
                            entity::get_component(player_id, user_id())
                                .unwrap_or_else(|| player_id.to_string())
                        ),
                    )
                    .spawn();
                entity::add_component(player_id, pc::character_ref(), character_id);
            }
        });

    // When a player despawns (leaves), despawn their character.
    despawn_query(pc::character_ref())
        .requires(is_player())
        .bind(|players| {
            for (_, character_ref) in players {
                entity::despawn_recursive(character_ref);
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
                .with(pc::input_sprint(), input.sprint)
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

    // Sync player input state to character input state.
    query((
        pc::input_direction(),
        pc::input_jump(),
        pc::input_fire(),
        pc::input_sprint(),
        pc::input_aim_direction(),
        pc::character_ref(),
    ))
    .each_frame(|players| {
        for (
            _,
            (
                input_direction,
                input_jump,
                input_fire,
                input_sprint,
                input_aim_direction,
                character_id,
            ),
        ) in players
        {
            if !entity::exists(character_id) {
                return;
            }

            entity::add_component(
                character_id,
                uc::run_direction(),
                vec2(input_direction.y, input_direction.x),
            );
            entity::add_component(character_id, uc::running(), input_sprint);
            entity::add_component(character_id, uc::shooting(), input_fire);
            entity::add_component(
                character_id,
                rotation(),
                Quat::from_rotation_z(input_aim_direction.x),
            );
            if let Some(head) = entity::get_component(character_id, uc::head_ref()) {
                entity::set_component(
                    head,
                    rotation(),
                    Quat::from_rotation_y(input_aim_direction.y)
                        * Quat::from_rotation_z(PI / 2.)
                        * Quat::from_rotation_x(PI / 2.),
                );
            }

            if input_jump {
                if entity::get_component(character_id, uc::is_on_ground()).unwrap_or_default() {
                    entity::add_component(character_id, uc::vertical_velocity(), 0.1);
                    entity::add_component(character_id, uc::jumping(), true);
                } else {
                    entity::add_component(character_id, uc::jumping(), false);
                }
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

    sp.translation + ((random::<Vec2>() - 0.5) * 2.0 * sp.radius).extend(0.0)
}
