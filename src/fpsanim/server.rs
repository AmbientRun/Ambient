use std::process::Output;

use ambient_api::{
    animation::{AnimationNode, AnimationPlayer, BlendNode, PlayClipFromUrlNode},
    components::core::{
        animation::{apply_animation_player, blend},
        player::player,
    },
    entity::spawn,
    prelude::*,
};

fn calculate_blend_from_weight(weights: &[f32]) -> Vec<f32> {
    assert!(weights.len() >= 2);
    let mut blend = Vec::with_capacity(weights.len() - 1);
    let mut total = 0.0;
    for i in 0..weights.len() {
        total += weights[i];
    }
    // left weight is used to compare left and right
    let mut left_weight = weights[0] / total;
    for i in 0..weights.len() - 1 {
        left_weight += weights[i + 1] / total;
        let b: f32 = if left_weight != 0.0 {
            weights[i + 1] / left_weight
        } else {
            0.0
        };
        blend.push(b);
    }
    blend
}

fn add_anim_clip_and_blend_to_player(id: EntityId) {
    let walk_fd = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Walk Forward.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let walk_bk = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Walk Backward.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let walk_lt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Walk Left.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let walk_rt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Walk Right.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let walk_fd_lt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Walk Forward Left.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let walk_fd_rt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Walk Forward Right.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let walk_bk_lt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Walk Backward Left.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let walk_bk_rt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Walk Backward Right.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let run_fd = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Run Forward.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let run_bk = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Run Backward.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let run_lt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Run Left.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let run_rt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Run Right.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let run_fd_lt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Run Forward Left.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let run_fd_rt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Run Forward Right.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let run_bk_lt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Run Backward Left.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let run_bk_rt = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Run Backward Right.fbx/animations/mixamo.com.anim").unwrap(),
    );

    let idle = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Rifle Aiming Idle.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let blend1 = BlendNode::new(&walk_fd, &walk_bk, 0.0);
    let blend2 = BlendNode::new(&blend1, &walk_lt, 0.0);
    let blend3 = BlendNode::new(&blend2, &walk_rt, 0.0);
    let blend4 = BlendNode::new(&blend3, &walk_fd_lt, 0.0);
    let blend5 = BlendNode::new(&blend4, &walk_fd_rt, 0.0);
    let blend6 = BlendNode::new(&blend5, &walk_bk_lt, 0.0);
    let blend7 = BlendNode::new(&blend6, &walk_bk_rt, 0.0);
    let blend8 = BlendNode::new(&blend7, &run_fd, 0.0);
    let blend9 = BlendNode::new(&blend8, &run_bk, 0.0);
    let blend10 = BlendNode::new(&blend9, &run_lt, 0.0);
    let blend11 = BlendNode::new(&blend10, &run_rt, 0.0);
    let blend12 = BlendNode::new(&blend11, &run_fd_lt, 0.0);
    let blend13 = BlendNode::new(&blend12, &run_fd_rt, 0.0);
    let blend14 = BlendNode::new(&blend13, &run_bk_lt, 0.0);
    let blend15 = BlendNode::new(&blend14, &run_bk_rt, 0.0);
    let blend16 = BlendNode::new(&blend15, &idle, 0.0);
    entity::add_component(
        id,
        components::player_output_blend_node(),
        blend16.0.to_entity(),
    );
    entity::add_component(
        id,
        components::player_anim_blend(),
        vec![
            blend1.0.to_entity(),
            blend2.0.to_entity(),
            blend3.0.to_entity(),
            blend4.0.to_entity(),
            blend5.0.to_entity(),
            blend6.0.to_entity(),
            blend7.0.to_entity(),
            blend8.0.to_entity(),
            blend9.0.to_entity(),
            blend10.0.to_entity(),
            blend11.0.to_entity(),
            blend12.0.to_entity(),
            blend13.0.to_entity(),
            blend14.0.to_entity(),
            blend15.0.to_entity(),
            blend16.0.to_entity(),
        ],
    );
}

fn get_blend_node_for_playing(id: EntityId, index: usize) -> Option<BlendNode> {
    let node = entity::get_component(id, components::player_anim_blend());
    if node.is_none() {
        return None;
    }
    let node = node.unwrap();
    if node.len() <= index {
        return None;
    }
    let init_node = node[index];
    let node = AnimationNode::from_entity(init_node);
    let blend_node = BlendNode(node);
    return Some(blend_node);
}

fn set_blend_weight_on_entity(id: EntityId, blend_weights: Vec<f32>) {
    for (i, weight) in blend_weights.iter().enumerate() {
        let blend_node = get_blend_node_for_playing(id, i).unwrap();
        blend_node.set_weight(*weight);
    }
}

#[main]
pub fn main() {
    spawn_query((player(), components::player_model_ref())).bind(move |v| {
        for (id, (_, model)) in v {
            add_anim_clip_and_blend_to_player(id);
            let output_blend_node =
                entity::get_component(id, components::player_output_blend_node()).unwrap();
            let blend_node = BlendNode::from_entity(output_blend_node);
            let anim_player = AnimationPlayer::new(blend_node);
            entity::add_component(model, apply_animation_player(), anim_player.0);
            entity::add_component(id, components::player_jumping(), false);
        }
    });

    change_query((
        player(),
        components::player_health(),
        components::player_model_ref(),
        components::player_output_blend_node(),
    ))
    .track_change(components::player_health())
    .bind(move |res| {
        for (player_id, (_, health, model, output_node)) in res {
            if health <= 0 {
                let death = PlayClipFromUrlNode::new(
                    asset::url("assets/anim/Rifle Death.fbx/animations/mixamo.com.anim").unwrap(),
                );
                death.looping(false);

                let anim_player_entity =
                    entity::get_component(model, apply_animation_player()).unwrap();
                let anim_player = AnimationPlayer(anim_player_entity);
                anim_player.play(death);

                run_async(async move {
                    let clip = PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Rifle Death.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    );
                    clip.looping(false);
                    let dur = clip.clip_duration().await;
                    sleep(dur).await;
                    // after death animation, play the blend node again
                    let blend_node = BlendNode::from_entity(output_node);
                    anim_player.play(blend_node);
                });
            };
        }
    });

    change_query((
        player(),
        components::player_jumping(),
        components::player_model_ref(),
        components::player_output_blend_node(),
    ))
    .track_change(components::player_jumping())
    .bind(move |res| {
        for (player_id, (_, is_jumping, model, output_node)) in res {
            let anim_player_entity =
                entity::get_component(model, apply_animation_player()).unwrap();
            let anim_player = AnimationPlayer(anim_player_entity);
            if is_jumping {
                let clip = PlayClipFromUrlNode::new(
                    asset::url("assets/anim/Rifle Jump.fbx/animations/mixamo.com.anim").unwrap(),
                );
                clip.looping(false);

                anim_player.play(clip);
            } else {
                // let output_node = get_output_blend_node_for_playing(player_id).unwrap();
                let blend_node = BlendNode::from_entity(output_node);
                anim_player.play(blend_node);
            }
        }
    });
    query((
        player(),
        components::player_model_ref(),
        components::player_direction(),
        components::player_running(),
        components::player_health(),
        components::player_jumping(),
    ))
    .each_frame(move |res| {
        for (player_id, (_, _model, dir, is_running, health, jump)) in res {
            if health <= 0 {
                continue;
            }
            if jump {
                continue;
            }
            let mut weights = vec![0.0; 17];

            let fd = dir.y == -1.0;
            let bk = dir.y == 1.0;
            let lt = dir.x == -1.0;
            let rt = dir.x == 1.0;

            if !is_running {
                if fd && !lt && !rt {
                    weights[0] = 1.0;
                } else if bk && !lt && !rt {
                    weights[1] = 1.0;
                } else if lt && !fd && !bk {
                    weights[2] = 1.0;
                } else if rt && !fd && !bk {
                    weights[3] = 1.0;
                } else if fd && lt {
                    weights[4] = 1.0;
                } else if fd && rt {
                    weights[5] = 1.0;
                } else if bk && lt {
                    weights[6] = 1.0;
                } else if bk && rt {
                    weights[7] = 1.0;
                } else {
                    weights[16] = 1.0;
                }
            } else {
                if fd && !lt && !rt {
                    weights[8] = 1.0;
                } else if bk && !lt && !rt {
                    weights[9] = 1.0;
                } else if lt && !fd && !bk {
                    weights[10] = 1.0;
                } else if rt && !fd && !bk {
                    weights[11] = 1.0;
                } else if fd && lt {
                    weights[12] = 1.0;
                } else if fd && rt {
                    weights[13] = 1.0;
                } else if bk && lt {
                    weights[14] = 1.0;
                } else if bk && rt {
                    weights[15] = 1.0;
                } else {
                    weights[16] = 1.0;
                }
            }
            let blend_weight = calculate_blend_from_weight(&weights);
            set_blend_weight_on_entity(player_id, blend_weight);
        }
    });
}
