use afps_fpsanim::components;
use afps_schema::components::{
    player_direction, player_health, player_jumping, player_model_ref, player_running,
};
use ambient_api::{
    animation::{AnimationPlayer, BlendNode, PlayClipFromUrlNode},
    core::{
        animation::components::apply_animation_player,
        ecs::components::{children, parent},
        player::components::player,
    },
    prelude::*,
};

fn calculate_blend_from_weight(weights: &[f32]) -> Vec<f32> {
    assert!(weights.len() >= 2);
    let mut blend = Vec::with_capacity(weights.len() - 1);
    let total = weights.iter().sum::<f32>();
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
    let walk_fd = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Walk Forward.fbx/animations/mixamo.com.anim",
    ));
    let walk_bk = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Walk Backward.fbx/animations/mixamo.com.anim",
    ));
    let walk_lt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Walk Left.fbx/animations/mixamo.com.anim",
    ));
    let walk_rt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Walk Right.fbx/animations/mixamo.com.anim",
    ));
    let walk_fd_lt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Walk Forward Left.fbx/animations/mixamo.com.anim",
    ));
    let walk_fd_rt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Walk Forward Right.fbx/animations/mixamo.com.anim",
    ));
    let walk_bk_lt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Walk Backward Left.fbx/animations/mixamo.com.anim",
    ));
    let walk_bk_rt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Walk Backward Right.fbx/animations/mixamo.com.anim",
    ));
    let run_fd = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Run Forward.fbx/animations/mixamo.com.anim",
    ));
    let run_bk = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Run Backward.fbx/animations/mixamo.com.anim",
    ));
    let run_lt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Run Left.fbx/animations/mixamo.com.anim",
    ));
    let run_rt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Run Right.fbx/animations/mixamo.com.anim",
    ));
    let run_fd_lt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Run Forward Left.fbx/animations/mixamo.com.anim",
    ));
    let run_fd_rt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Run Forward Right.fbx/animations/mixamo.com.anim",
    ));
    let run_bk_lt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Run Backward Left.fbx/animations/mixamo.com.anim",
    ));
    let run_bk_rt = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Run Backward Right.fbx/animations/mixamo.com.anim",
    ));

    let idle = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
        "Rifle Aiming Idle.fbx/animations/mixamo.com.anim",
    ));
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
    let output = BlendNode::new(&blend16, &blend16, 0.0); // the right one is dummy
    entity::add_component(
        id,
        components::player_output_blend_node(),
        output.0.get_entity_id(),
    );

    entity::add_component(
        id,
        components::player_persistant_anim_node(),
        blend16.0.get_entity_id(),
    );

    entity::add_component(
        id,
        components::player_anim_blend(),
        vec![
            blend1.0.get_entity_id(),
            blend2.0.get_entity_id(),
            blend3.0.get_entity_id(),
            blend4.0.get_entity_id(),
            blend5.0.get_entity_id(),
            blend6.0.get_entity_id(),
            blend7.0.get_entity_id(),
            blend8.0.get_entity_id(),
            blend9.0.get_entity_id(),
            blend10.0.get_entity_id(),
            blend11.0.get_entity_id(),
            blend12.0.get_entity_id(),
            blend13.0.get_entity_id(),
            blend14.0.get_entity_id(),
            blend15.0.get_entity_id(),
            blend16.0.get_entity_id(),
        ],
    );
}

fn get_blend_node_for_playing(id: EntityId, index: usize) -> Option<BlendNode> {
    let node = entity::get_component(id, components::player_anim_blend())?;
    if node.len() <= index {
        return None;
    }
    let init_node = node[index];
    // let node = AnimationNode::from_entity(init_node);
    let blend_node = BlendNode::from_entity(init_node);
    Some(blend_node)
}

fn set_blend_weights_on_entity(id: EntityId, blend_weights: Vec<f32>) {
    for (i, weight) in blend_weights.iter().enumerate() {
        let blend_node = get_blend_node_for_playing(id, i).unwrap();
        blend_node.set_weight(*weight);
    }
}

#[main]
pub fn main() {
    spawn_query((player(), player_model_ref())).bind(move |v| {
        for (id, (_, model)) in v {
            add_anim_clip_and_blend_to_player(id);
            let output_blend_node =
                entity::get_component(id, components::player_output_blend_node()).unwrap();
            let blend_node = BlendNode::from_entity(output_blend_node);
            let anim_player = AnimationPlayer::new(blend_node);
            entity::add_component(model, apply_animation_player(), anim_player.0);
            entity::add_component(id, player_jumping(), false);
        }
    });

    change_query((
        player(),
        player_health(),
        components::player_output_blend_node(),
        components::player_persistant_anim_node(),
    ))
    .track_change(player_health())
    .bind(move |res| {
        for (_, (_, health, output_node, persistant_node)) in res {
            if health <= 0 {
                let clip = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
                    "Rifle Death.fbx/animations/mixamo.com.anim",
                ));
                clip.looping(false);

                entity::mutate_component(output_node, children(), |v| {
                    v.clear();
                    v.push(clip.0.get_entity_id());
                    v.push(clip.0.get_entity_id());
                });
                entity::add_component(clip.0.get_entity_id(), parent(), output_node);

                run_async(async move {
                    let clip = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
                        "Rifle Death.fbx/animations/mixamo.com.anim",
                    ));
                    clip.looping(false);
                    let dur = clip.clip_duration().await;
                    sleep(dur).await;
                    // after death animation, play the blend node again
                    entity::mutate_component(output_node, children(), |v| {
                        v.clear();
                        v.push(persistant_node);
                        v.push(persistant_node);
                    });
                    entity::add_component(persistant_node, parent(), output_node);
                });
            };
        }
    });

    change_query((
        player(),
        player_jumping(),
        components::player_output_blend_node(),
        components::player_persistant_anim_node(),
    ))
    .track_change(player_jumping())
    .bind(move |res| {
        for (_, (_, is_jumping, output_node, persistant_node)) in res {
            if is_jumping {
                let clip = PlayClipFromUrlNode::new(afps_fpsanim::assets::url(
                    "Rifle Jump.fbx/animations/mixamo.com.anim",
                ));
                clip.looping(false);
                entity::mutate_component(output_node, children(), |v| {
                    v.clear();
                    v.push(clip.0.get_entity_id());
                    v.push(clip.0.get_entity_id());
                });
                entity::add_component(clip.0.get_entity_id(), parent(), output_node);
            } else {
                entity::mutate_component(output_node, children(), |v| {
                    v.clear();
                    v.push(persistant_node);
                    v.push(persistant_node); // we had to add the second one for blend node to work
                });
                entity::add_component(persistant_node, parent(), output_node);
            }
        }
    });
    query((
        player(),
        player_model_ref(),
        player_direction(),
        player_running(),
        player_health(),
        player_jumping(),
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
            } else if fd && !lt && !rt {
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
            let blend_weight = calculate_blend_from_weight(&weights);
            set_blend_weights_on_entity(player_id, blend_weight);
        }
    });
}
