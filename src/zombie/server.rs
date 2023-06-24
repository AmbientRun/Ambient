#[allow(unused_imports)]
use ambient_api::{
    animation::{get_bone_by_bind_id, AnimationPlayer, BindId, BlendNode, PlayClipFromUrlNode},
    components::core::{
        animation::apply_animation_player,
        app::main_scene,
        camera::aspect_ratio_from_window,
        ecs::{children, parent},
        physics::{
            character_controller_height, character_controller_radius, cube_collider, dynamic,
            linear_velocity, physics_controlled, plane_collider, sphere_collider,
        },
        player::{player, user_id},
        prefab::prefab_from_url,
        primitives::{cube, quad},
        rendering::color,
        transform::{local_to_parent, rotation, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};
#[main]
pub async fn main() {
    let chars = vec![
        asset::url("assets/model/Zombiegirl W Kurniawan.fbx").unwrap(),
        asset::url("assets/model/copzombie_l_actisdato.fbx").unwrap(),
        asset::url("assets/model/Yaku J Ignite.fbx").unwrap(),
    ];

    run_async(async move {
        for i in 0..3 {
            let zombie = Entity::new().spawn();

            let model = make_transformable()
                .with(prefab_from_url(), chars[i].clone())
                .with(parent(), zombie)
                .with_default(local_to_parent())
                .with(rotation(), Quat::from_rotation_z(-3.14159265359 / 2.0))
                // .with_default(local_to_world())
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
                    .with_default(local_to_world())
                    .with_default(physics_controlled())
                    .with(components::zombie_model_ref(), model)
                    .with(components::zombie_health(), 100)
                    .with_default(components::is_zombie()),
            );

            let run = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Zombie Run.fbx/animations/mixamo.com.anim").unwrap(),
            );

            let blend = BlendNode::new(&run, &run, 0.);
            let anim_player = AnimationPlayer::new(&blend);
            entity::add_component(model, apply_animation_player(), anim_player.0);

            sleep(random::<f32>()).await;
        }
    });

    let player_query = query(translation()).requires(player()).build();

    query((translation(), components::is_zombie())).each_frame(move |zombies| {
        for (zombie, (pos, _)) in zombies {
            let players: Vec<(EntityId, Vec3)> = player_query.evaluate();
            let zombie_pos = vec2(pos.x, pos.y);

            let mut min_distance = std::f32::MAX;
            let mut nearest_player_pos: Option<Vec2> = None;

            for (_player, pos) in players {
                // println!("player pos {:?}", pos);
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
                    frametime(),
                );
                entity::set_component(zombie, rotation(), rot);
                // println!("collision: {} {} {}", collision.up, collision.down, collision.side);
            }
        }
    });
}
