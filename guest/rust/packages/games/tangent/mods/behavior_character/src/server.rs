use std::f32::consts::PI;

use ambient_api::{
    core::{
        app::components::name,
        model::components::model_from_url,
        physics::concepts::CharacterController,
        transform::{
            components::{local_to_parent, rotation, translation},
            concepts::{Transformable, TransformableOptional},
        },
    },
    prelude::*,
};
use packages::{
    character_animation::components::basic_character_animations,
    tangent_schema::{
        character::components::is_character,
        concepts::{Character, CharacterDef},
    },
    unit_schema::components as uc,
};

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
                    .with_merge(CharacterController {
                        character_controller_height: 2.,
                        character_controller_radius: 0.5,
                        physics_controlled: (),
                    })
                    .with_merge(Transformable {
                        local_to_world: default(),
                        optional: TransformableOptional {
                            translation: None,
                            rotation: Some(Quat::IDENTITY),
                            scale: None,
                        },
                    })
                    .with(uc::speed(), def.speed)
                    .with(uc::run_speed_multiplier(), def.run_speed_multiplier)
                    .with(uc::strafe_speed_multiplier(), def.strafe_speed_multiplier)
                    .with(uc::run_direction(), Vec2::ZERO)
                    .with(uc::vertical_velocity(), 0.)
                    .with(uc::running(), false)
                    .with(uc::jumping(), false),
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
