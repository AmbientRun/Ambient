#[allow(unused_imports)]
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

mod anim;

#[main]
pub fn main() {
    anim::register_anim();
    query((player(), components::player_direction())).each_frame(|results| {
        for (player_id, (_, dir)) in results {
            let fd = dir.y == -1.0;
            let bk = dir.y == 1.0;
            let lt = dir.x == -1.0;
            let rt = dir.x == 1.0;

            if fd {
                apply_anim(player_id, components::idle_fd(), 1.0);
            } else if bk {
                apply_anim(player_id, components::idle_bk(), 1.0);
            } else if lt {
                apply_anim(player_id, components::idle_lt(), 1.0);
            } else if rt {
                apply_anim(player_id, components::idle_rt(), 1.0);
            } else if fd && lt {
                apply_anim(player_id, components::fd_lt(), 0.5);
            } else if fd && rt {
                apply_anim(player_id, components::fd_rt(), 0.5);
            } else if bk && lt {
                apply_anim(player_id, components::bk_lt(), 0.5);
            } else if bk && rt {
                apply_anim(player_id, components::bk_rt(), 0.5);
            } else {
                apply_anim(player_id, components::idle_bk(), 0.0);
            }
        }
    });
}

pub fn apply_anim(player_id: EntityId, comp: Component<Vec<EntityId>>, blend_value: f32) {
    let model = entity::get_component(player_id, components::player_model_ref()).unwrap();
    let blend_player = entity::get_component(entity::resources(), comp).unwrap();
    entity::set_component(blend_player[0], blend(), blend_value);
    entity::add_component(model, apply_animation_player(), blend_player[1]);
}
