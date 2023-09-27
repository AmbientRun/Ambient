use std::{f32::consts::PI, sync::OnceLock};

use ambient_api::{
    core::{
        app::components::name,
        player::components::{is_player, user_id},
        transform::components::{rotation, scale, translation},
    },
    ecs::GeneralQuery,
    prelude::*,
};

use packages::{
    game_object::{components as goc, player::components as gopc},
    nameplates::components::height_offset,
    tangent_schema::{
        character::components as cc,
        concepts::{Character, CharacterDef, PlayerClass, Spawnpoint},
        player::components as pc,
        vehicle::components as vc,
    },
    this::messages::{Input, UseFailed},
    unit_schema::components as uc,
};

#[main]
pub fn main() {
    // When the player's class changes, respawn them.
    change_query(pc::class_ref())
        .track_change(pc::class_ref())
        .requires(is_player())
        .bind(move |players| {
            for (player_id, class_id) in players {
                respawn_player(player_id, class_id);
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

    // If the player's character is dead, respawn them.
    change_query((cc::player_ref(), goc::health()))
        .track_change(goc::health())
        .requires(cc::is_character())
        .bind(|characters| {
            for (_character_id, (player_id, health)) in characters {
                if health > 0.0 {
                    continue;
                }

                let Some(class_id) = entity::get_component(player_id, pc::class_ref()) else {
                    continue;
                };

                respawn_player(player_id, class_id);
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

                    let Some(character_id) = entity::get_component(driver_id, pc::character_ref())
                    else {
                        continue;
                    };

                    entity::add_components(
                        character_id,
                        Entity::new()
                            .with(uc::run_direction(), Vec2::ZERO)
                            .with(uc::running(), false)
                            .with(uc::shooting(), false)
                            .with(scale(), Vec3::ONE * 0.01),
                    );
                }
            });

        despawn_query(vc::driver_ref()).bind(|vehicles| {
            for (_vehicle_id, driver_id) in vehicles {
                entity::remove_component(driver_id, pc::vehicle_ref());

                let Some(character_id) = entity::get_component(driver_id, pc::character_ref())
                else {
                    continue;
                };
                entity::add_component(driver_id, gopc::control_of_entity(), character_id);
                entity::set_component(character_id, scale(), Vec3::ONE);
            }
        });
    }

    // Move characters with their vehicles (parenting of character controllers doesn't work)
    query((pc::character_ref(), pc::vehicle_ref())).each_frame(|players| {
        for (_, (character_ref, vehicle_ref)) in players {
            if !entity::exists(vehicle_ref) {
                continue;
            }

            let Some(vehicle_translation) = entity::get_component(vehicle_ref, translation())
            else {
                continue;
            };

            let Some(vehicle_rotation) = entity::get_component(vehicle_ref, rotation()) else {
                continue;
            };

            entity::set_component(
                character_ref,
                translation(),
                vehicle_translation + vehicle_rotation * Vec3::Z,
            );
            entity::set_component(
                character_ref,
                rotation(),
                Quat::from_rotation_z(-90f32.to_radians()) * vehicle_rotation,
            );
        }
    });

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
                .with(pc::input_use(), input.use_button)
                .with(pc::input_sprint(), input.sprint)
                .with(pc::input_respawn(), input.respawn)
                .with(pc::input_aim_direction(), input.aim_direction)
                .with(pc::input_aim_ray_origin(), input.aim_ray_origin)
                .with(pc::input_aim_ray_direction(), input.aim_ray_direction),
        );
    });

    // Sync player input state to vehicle input state.
    query((
        pc::input_direction(),
        pc::input_jump(),
        pc::input_fire(),
        pc::input_respawn(),
        pc::input_aim_ray_origin(),
        pc::input_aim_ray_direction(),
        pc::vehicle_ref(),
    ))
    .each_frame(|players| {
        for (
            player_id,
            (
                input_direction,
                input_jump,
                input_fire,
                input_respawn,
                aim_ray_origin,
                aim_ray_direction,
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

            entity::add_components(
                vehicle_id,
                Entity::new()
                    .with(vc::input_direction(), input_direction)
                    .with(vc::input_jump(), input_jump)
                    .with(vc::input_fire(), input_fire),
            );

            let hit = physics::raycast(aim_ray_origin, aim_ray_direction)
                .into_iter()
                .find(|hit| {
                    !(entity::get_component(hit.entity, cc::player_ref()) == Some(player_id)
                        || entity::get_component(hit.entity, vc::driver_ref()) == Some(player_id))
                });

            if let Some(hit) = hit {
                entity::add_component(vehicle_id, vc::aim_position(), hit.position);
            }
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
    .excludes(pc::vehicle_ref())
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

            entity::add_components(
                character_id,
                Entity::new()
                    .with(
                        uc::run_direction(),
                        vec2(input_direction.y, input_direction.x),
                    )
                    .with(uc::running(), input_sprint)
                    .with(uc::shooting(), input_fire)
                    .with(rotation(), Quat::from_rotation_z(input_aim_direction.x)),
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

    // Handle use key
    query((
        pc::character_ref(),
        pc::input_aim_ray_origin(),
        pc::input_aim_ray_direction(),
        pc::input_use(),
    ))
    .each_frame(|players| {
        const MAX_USE_DISTANCE: f32 = 3.0;
        const MAX_USE_DISTANCE_SQR: f32 = MAX_USE_DISTANCE * MAX_USE_DISTANCE;

        for (player_id, (character_ref, ray_origin, ray_direction, input_use)) in players {
            if !input_use {
                continue;
            }

            let last_use_time =
                entity::get_component(character_ref, cc::last_use_time()).unwrap_or_default();

            let character_translation = entity::get_component(character_ref, translation())
                .unwrap_or(ray_origin + 2.0 * ray_direction);

            if (game_time() - last_use_time) < Duration::from_secs_f32(0.5) {
                continue;
            }

            match entity::get_component(player_id, pc::vehicle_ref()) {
                Some(vehicle_id) => {
                    // Remove the driving component `driver_ref` so that `vehicle_ref` is updated
                    entity::remove_component(vehicle_id, vc::driver_ref());
                }
                _ => {
                    let hit = physics::raycast(ray_origin, ray_direction)
                        .into_iter()
                        .find(|h| {
                            h.entity != character_ref
                                && h.position.distance_squared(character_translation)
                                    < MAX_USE_DISTANCE_SQR
                        });

                    match hit {
                        Some(hit) if entity::has_component(hit.entity, vc::is_vehicle()) => {
                            entity::add_component(hit.entity, vc::driver_ref(), player_id);
                        }
                        _ => {
                            UseFailed.send_client_targeted_reliable(
                                entity::get_component(player_id, user_id()).unwrap(),
                            );
                        }
                    }
                }
            }
            entity::add_component(character_ref, cc::last_use_time(), game_time());
        }
    });
}

fn respawn_player(player_id: EntityId, class_id: EntityId) {
    if let Some(character_ref) = entity::get_component(player_id, pc::character_ref()) {
        entity::despawn_recursive(character_ref);
    }

    let Some(class) = PlayerClass::get_spawned(class_id) else {
        return;
    };

    let Some(def) = CharacterDef::get_spawned(class.def_ref) else {
        return;
    };

    let character_id = Character {
        translation: choose_spawn_position(),
        rotation: Quat::IDENTITY,
        health: def.max_health,
        max_health: def.max_health,
        is_character: (),
        player_ref: player_id,
        def_ref: class.def_ref,
        optional: default(),
    }
    .make()
    .with(height_offset(), 2.0)
    .with(
        name(),
        format!(
            "{}'s Character",
            entity::get_component(player_id, user_id()).unwrap_or_else(|| player_id.to_string())
        ),
    )
    .spawn();
    entity::add_component(player_id, pc::character_ref(), character_id);
    entity::add_component(player_id, gopc::control_of_entity(), character_id);
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
