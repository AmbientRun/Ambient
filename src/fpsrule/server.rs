use ambient_api::{
    animation::{AnimationPlayer, BlendNode, PlayClipFromUrlNode},
    components::core::{
        animation::{apply_animation_player, blend},
        app::main_scene,
        camera::aspect_ratio_from_window,
        ecs::{children, parent},
        physics::{
            character_controller_height, character_controller_radius, physics_controlled,
            plane_collider, sphere_collider,
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

// use components::{player_head_ref, player_movement_direction, player_pitch, player_yaw};
use std::f32::consts::{E, PI, TAU};

const MAX_SPEED: f32 = 0.1;
const SPEED_DELTA: f32 = 0.01;

#[main]
pub fn main() {
    spawn_query(player()).bind(move |players| {
        for (id, _) in players {
            entity::add_component(id, components::player_health(), 100);
            entity::add_component(id, components::hit_freeze(), 0);
        }
    });

    messages::Shoot::subscribe(move |_source, msg| {
        let result = physics::raycast_first(msg.ray_origin, msg.ray_dir);
        // let game_type =
        //     entity::get_component(entity::resources(), components::game_type()).unwrap();
        if let Some(hit) = result {
            if entity::has_component(hit.entity, components::player_health()) {
                let old_health =
                    entity::get_component(hit.entity, components::player_health()).unwrap();
                println!("hit player: {}", old_health);
                if old_health <= 0 {
                    return;
                }
                let new_health = (old_health - 10).max(0);
                entity::set_component(hit.entity, components::player_health(), new_health);
                // let model =
                //     entity::get_component(hit.entity, components::player_model_ref()).unwrap();

                if old_health > 0 && new_health <= 0 {
                    println!("player death");
                    // let death_anim =
                    //     entity::get_component(entity::resources(), components::death()).unwrap()[1];
                    // entity::set_component(model, apply_animation_player(), death_anim);
                } else {
                    entity::set_component(hit.entity, components::hit_freeze(), 20);
                    // let hit_anim =
                    //     entity::get_component(entity::resources(), components::hit()).unwrap()[1];
                    // entity::set_component(model, apply_animation_player(), hit_anim);
                }
            }
        }
    });
}
