use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    core::{
        app::components::name,
        ecs::components::{children, parent},
        physics::concepts::make_CharacterController,
        player::components::is_player,
        prefab::components::prefab_from_url,
        transform::{
            components::{local_to_parent, local_to_world, rotation, translation},
            concepts::make_Transformable,
        },
    },
    prelude::*,
};

use crate::packages::{
    character_animation,
    unit_schema::components::{running, speed, vertical_velocity},
};
use packages::unit_schema::components::{health, run_direction};
use packages::{
    character_animation::components::basic_character_animations,
    this::{assets, components},
};

#[main]
pub async fn main() {
    let chars = [
        assets::url("Zombiegirl W Kurniawan.fbx"),
        assets::url("copzombie_l_actisdato.fbx"),
        assets::url("Yaku J Ignite.fbx"),
    ];
    fn anim_url(name: &str) -> String {
        assets::url(&format!("{name}.fbx/animations/mixamo.com.anim"))
    }

    run_async(async move {
        for character_url in chars {
            let zombie = Entity::new().spawn();

            let model = make_Transformable()
                .with(prefab_from_url(), character_url)
                .with(parent(), zombie)
                .with(local_to_parent(), Default::default())
                .with(rotation(), Quat::from_rotation_z(-FRAC_PI_2))
                .spawn();

            entity::add_components(
                zombie,
                make_Transformable()
                    .with_merge(make_CharacterController())
                    .with(
                        translation(),
                        vec3(-8.0 * random::<f32>(), -8.0 * random::<f32>(), 1.3),
                    )
                    .with(name(), "Zombie".to_string())
                    .with(children(), vec![model])
                    .with(local_to_world(), Default::default())
                    .with(components::zombie_model_ref(), model)
                    .with(health(), 100.)
                    .with(run_direction(), -Vec2::Y)
                    .with(vertical_velocity(), 0.)
                    .with(speed(), 0.03)
                    .with(running(), false)
                    .with(components::is_zombie(), ())
                    .with(basic_character_animations(), model)
                    .with(
                        character_animation::components::idle(),
                        anim_url("Zombie Idle"),
                    )
                    .with(
                        character_animation::components::walk_forward(),
                        anim_url("Zombie Walk"),
                    )
                    .with(
                        character_animation::components::run_forward(),
                        anim_url("Zombie Run"),
                    )
                    .with(
                        character_animation::components::death(),
                        anim_url("Zombie Death"),
                    ),
            );

            sleep(random::<f32>()).await;
        }
    });

    let player_query = query(translation()).requires(is_player()).build();

    query((translation(), components::is_zombie())).each_frame(move |zombies| {
        for (zombie, (pos, _)) in zombies {
            let players: Vec<(EntityId, Vec3)> = player_query.evaluate();
            let zombie_pos = vec2(pos.x, pos.y);

            let mut min_distance = 5.;
            let mut nearest_player_pos: Option<Vec2> = None;

            for (_player, pos) in players {
                let player_pos = vec2(pos.x, pos.y);
                let distance = (zombie_pos - player_pos).length();
                if distance < min_distance {
                    min_distance = distance;
                    nearest_player_pos = Some(player_pos);
                }
            }

            if let Some(nearest_player_pos) = nearest_player_pos {
                let dir = (nearest_player_pos - zombie_pos).normalize_or_zero();

                let angle = dir.y.atan2(dir.x);
                let rot = Quat::from_rotation_z(angle);
                entity::set_component(zombie, rotation(), rot);
                entity::set_component(zombie, run_direction(), Vec2::X);
                // println!("collision: {} {} {}", collision.up, collision.down, collision.side);
            } else {
                entity::set_component(zombie, run_direction(), Vec2::ZERO);
            }
        }
    });
}
