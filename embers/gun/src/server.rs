use ambient_api::{
    animation::{get_bone_by_bind_id, BindId},
    animation::{AnimationPlayer, BlendNode, PlayClipFromUrlNode},
    core::{
        animation::components::apply_animation_player,
        ecs::components::{children, parent},
        physics::components::{
            character_controller_height, character_controller_radius, physics_controlled,
        },
        player::components::player,
        prefab::components::prefab_from_url,
        transform::{
            components::{
                local_to_parent, local_to_world, reset_scale, rotation, scale, translation,
            },
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use afps_gun::components;
use afps_gun::messages;

#[main]
pub async fn main() {
    println!("Hello, guns!!!!_______");
    for i in 0..100 {
        Entity::new()
            .with_merge(make_transformable())
            .with(prefab_from_url(), {
                if i % 3 == 0 {
                    asset::url("afps_fpsmodel/assets/red.glb").unwrap()
                } else if i % 3 == 1 {
                    asset::url("afps_fpsmodel/assets/green.glb").unwrap()
                } else {
                    asset::url("afps_fpsmodel/assets/blue.glb").unwrap()
                }
            })
            // .with(
            // rotation(),
            // Quat::from_rotation_x(std::f32::consts::PI * random())
            // Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)
            // * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            // )
            .with(translation(), vec3(i as f32 * 2.0, 0.0, 0.2))
            .with(scale(), Vec3::ONE * 0.3)
            .with(components::is_gun(), (i % 3) as u8)
            .spawn();
    }

    let player_query = query(translation()).requires(player()).build();
    query((components::is_gun(), translation())).each_frame(move |entities| {
        for (e, (gun_type, cm_pos)) in entities {
            let players: Vec<(EntityId, Vec3)> = player_query.evaluate();
            for (player_id, player_pos) in players {
                // let player_pos = vec2(pos.x, pos.y);
                let distance = (cm_pos - player_pos).length();

                if distance < 0.5 {
                    println!("should switch gun on this player");
                    entity::despawn(e);
                    messages::Switch::new(player_id, gun_type).send_client_broadcast_reliable();
                }
            }
        }
    });
}
