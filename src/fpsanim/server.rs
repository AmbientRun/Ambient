use ambient_api::{
    animation::{AnimationPlayer, AnimationRetargeting, BlendNode, PlayClipFromUrlNode},
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

#[derive(Debug, Clone)]
struct FPSAnimBlend {
    // clips: Vec<PlayClipFromUrlNode>,
    pub nodes: Vec<BlendNode>,
    // pub output: BlendNode,
}

impl FPSAnimBlend {
    pub fn new() -> Self {
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

        Self {
            nodes: vec![
                blend1, blend2, blend3, blend4, blend5, blend6, blend7, blend8, blend9, blend10,
                blend11, blend12, blend13, blend14, blend15, blend16,
            ],
        }
    }
    pub fn update_weights(&mut self, weights: &[f32]) {
        let blend = calculate_blend_from_weight(weights);
        println!("current frame blend{:?}", blend);
        for i in 0..self.nodes.len() {
            self.nodes[i].set_weight(blend[i]);
        }
    }
}

#[main]
pub fn main() {
    let anim_lib = std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()));
    let anim_lib_clone = std::rc::Rc::clone(&anim_lib);
    let anim_lib_once = std::rc::Rc::clone(&anim_lib);

    spawn_query((player(), components::player_model_ref())).bind(move |v| {
        for (id, (_, model)) in v {
            let fps_blend = FPSAnimBlend::new();
            let anim_player = AnimationPlayer::new(fps_blend.nodes.last().unwrap());
            anim_lib.borrow_mut().insert(id, (fps_blend, anim_player));
            entity::add_component(model, apply_animation_player(), anim_player.0);
            entity::add_component(id, components::player_jumping(), false);
        }
    });
    change_query((player(), components::player_jumping()))
        .track_change(components::player_jumping())
        .bind(move |res| {
            for (player_id, (_, is_jumping)) in res {
                let anim_lib = anim_lib_once.borrow_mut();
                let (blend, anim_player) = anim_lib.get(&player_id).unwrap().clone();
                if is_jumping {
                    let clip = PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Rifle Jump.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    );
                    clip.looping(false);

                    anim_player.play(clip);

                    run_async(async move {
                        let clip = PlayClipFromUrlNode::new(
                            asset::url("assets/anim/Rifle Jump.fbx/animations/mixamo.com.anim")
                                .unwrap(),
                        );
                        clip.looping(false);
                        let dur = clip.clip_duration().await;
                        std::thread::sleep(std::time::Duration::from_secs_f32(dur));
                        let another_clip = PlayClipFromUrlNode::new(
                            asset::url("assets/anim/Rifle Death.fbx/animations/mixamo.com.anim")
                                .unwrap(),
                        );
                        another_clip.looping(false);
                        anim_player.play(another_clip);
                    });
                };

                //  else {
                //     anim_player.play(blend.nodes.last().unwrap());
                // }
            }
        });
    query((
        player(),
        components::player_model_ref(),
        components::player_direction(),
        components::player_running(),
    ))
    .each_frame(move |res| {
        for (player_id, (_, _model, dir, is_running)) in res {
            let mut weights = vec![0.0; 17];
            let anim_lib = anim_lib_clone.borrow_mut();
            let (mut blend, _anim_player) = anim_lib.get(&player_id).unwrap().clone();
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
            blend.update_weights(&weights);
            println!("current frame weight{:?}", weights);
        }
    });
}
