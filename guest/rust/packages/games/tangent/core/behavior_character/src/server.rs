use std::f32::consts::PI;

use ambient_api::{
    core::{
        app::components::name,
        model::components::model_from_url,
        transform::{
            components::{local_to_parent, local_to_world, rotation, translation},
            concepts::Transformable,
        },
    },
    prelude::*,
};
use packages::{
    character_animation::components::basic_character_animations,
    character_movement::concepts::{CharacterMovement, CharacterMovementOptional},
    tangent_schema::{
        character::components::is_character,
        concepts::{Character, CharacterDef},
    },
    unit_schema::components as uc,
};

pub mod packages;

#[main]
pub fn main() {
    spawn_query(Character::as_query()).bind(move |characters| {
        for (id, character) in characters {
            let Some(def) = CharacterDef::get_spawned(character.def_ref) else {
                continue;
            };

            entity::add_components(
                id,
                Entity::new()
                    .with(model_from_url(), def.model_url)
                    .with(basic_character_animations(), id)
                    .with(local_to_world(), default())
                    .with_merge(CharacterMovement {
                        character_controller_height: 2.,
                        character_controller_radius: 0.5,
                        physics_controlled: (),
                        rotation: Quat::IDENTITY,
                        run_direction: Vec2::ZERO,
                        vertical_velocity: 0.,
                        running: false,
                        jumping: false,
                        is_on_ground: true,
                        optional: CharacterMovementOptional {
                            run_speed_multiplier: Some(def.run_speed_multiplier),
                            speed: Some(def.speed),
                            strafe_speed_multiplier: Some(def.strafe_speed_multiplier),
                            air_speed_multiplier: Some(1.0),
                        },
                    }),
            );
        }
    });

    spawn_query(is_character())
        .excludes(uc::head_ref())
        .bind(|characters| {
            for (id, _) in characters {
                let head = Entity::new()
                    .with(name(), "Head".to_string())
                    .with_merge(Transformable {
                        local_to_world: default(),
                        optional: default(),
                    })
                    .with(local_to_parent(), Default::default())
                    .with(translation(), Vec3::Z * 2.)
                    .with(
                        rotation(),
                        Quat::from_rotation_z(PI / 2.) * Quat::from_rotation_x(PI / 2.),
                    )
                    .spawn();
                entity::add_child(id, head);
                entity::add_component(id, uc::head_ref(), head);
            }
        });
}
