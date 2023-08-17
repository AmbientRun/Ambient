use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    animation::{AnimationPlayer, BlendNode, PlayClipFromUrlNode},
    core::{
        animation::components::apply_animation_player,
        ecs::components::{children, parent},
        physics::components::{
            character_controller_height, character_controller_radius, physics_controlled,
        },
        player::components::is_player,
        prefab::components::prefab_from_url,
        transform::{
            components::{local_to_parent, local_to_world, rotation, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use packages::afps_zombie::{assets, components};

#[main]
pub async fn main() {
    let chars = [
        assets::url("Zombiegirl W Kurniawan.fbx"),
        assets::url("copzombie_l_actisdato.fbx"),
        assets::url("Yaku J Ignite.fbx"),
    ];

    run_async(async move {
        for character_url in chars {
            let zombie = Entity::new().spawn();

            let model = make_transformable()
                .with(prefab_from_url(), character_url)
                .with(parent(), zombie)
                .with(local_to_parent(), Default::default())
                .with(rotation(), Quat::from_rotation_z(-FRAC_PI_2))
                .spawn();

            entity::add_components(
                zombie,
                make_transformable()
                    .with(character_controller_height(), 2.)
                    .with(character_controller_radius(), 0.5)
                    .with(
                        translation(),
                        vec3(-8.0 * random::<f32>(), -8.0 * random::<f32>(), 1.3),
                    )
                    .with(children(), vec![model])
                    .with(local_to_world(), Default::default())
                    .with(physics_controlled(), ())
                    .with(components::zombie_model_ref(), model)
                    .with(components::zombie_health(), 100)
                    .with(components::is_zombie(), ()),
            );

            let run =
                PlayClipFromUrlNode::new(assets::url("Zombie Run.fbx/animations/mixamo.com.anim"));

            let blend = BlendNode::new(&run, &run, 0.);
            let anim_player = AnimationPlayer::new(&blend);
            entity::add_component(model, apply_animation_player(), anim_player.0);

            sleep(random::<f32>()).await;
        }
    });

    let player_query = query(translation()).requires(is_player()).build();

    query((translation(), components::is_zombie())).each_frame(move |zombies| {
        for (zombie, (pos, _)) in zombies {
            let players: Vec<(EntityId, Vec3)> = player_query.evaluate();
            let zombie_pos = vec2(pos.x, pos.y);

            let mut min_distance = std::f32::MAX;
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
                let displace = nearest_player_pos - zombie_pos;
                // if displace.length() > 5.0 {
                //     break;
                // }
                let zb_speed = 0.03;
                // If you want the zombie to move at a constant speed regardless of distance to the player,
                // you may want to normalize the displacement vector before feeding it to `move_character`
                let displace = displace.normalize_or_zero() * zb_speed; // normalize to get a unit vector

                let angle = displace.y.atan2(displace.x);
                let rot = Quat::from_rotation_z(angle);
                let _collision = physics::move_character(
                    zombie,
                    vec3(displace.x, displace.y, -0.1),
                    0.01,
                    delta_time(),
                );
                entity::set_component(zombie, rotation(), rot);
                // println!("collision: {} {} {}", collision.up, collision.down, collision.side);
            }
        }
    });
}
