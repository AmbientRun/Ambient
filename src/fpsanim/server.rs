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
    let mut left_weight = weights[0];
    for i in 0..weights.len() - 1 {
        left_weight += weights[i + 1];
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
        let jump = PlayClipFromUrlNode::new(
            asset::url("assets/anim/Rifle Jump.fbx/animations/mixamo.com.anim").unwrap(),
        );
        jump.looping(false);
        let idle = PlayClipFromUrlNode::new(
            asset::url("assets/anim/Rifle Aiming Idle.fbx/animations/mixamo.com.anim").unwrap(),
        );
        let blend1 = BlendNode::new(&walk_fd, &jump, 0.5);
        let blend2 = BlendNode::new(&blend1, &idle, 0.5);

        Self {
            nodes: vec![blend1, blend2],
        }
    }
    pub fn update_weights(&mut self, weights: &[f32]) {
        let blend = calculate_blend_from_weight(weights);
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
                let (mut blend, anim_player) = anim_lib.get(&player_id).unwrap().clone();
                if is_jumping {
                    let clip = PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Rifle Jump.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    );
                    clip.looping(false);
                    anim_player.play(clip);
                } else {
                    anim_player.play(blend.nodes.last().unwrap());
                }
            }
        });
    query((
        player(),
        components::player_model_ref(),
        // components::player_jumping(),
        components::player_direction(),
    ))
    .each_frame(move |res| {
        for (player_id, (_, _model, dir)) in res {
            let mut weights = vec![1.0, 0.0, 0.0];
            let anim_lib = anim_lib_clone.borrow_mut();
            let (mut blend, _anim_player) = anim_lib.get(&player_id).unwrap().clone();

            if dir.x == 0.0 && dir.y == 0.0 {
                //idle
                weights = vec![0.0, 0.0, 1.0];
            } else {
                // walk
                weights = vec![1.0, 0.0, 0.0];
            }
            // }
            blend.update_weights(&weights);
            println!("current frame weight{:?}", weights);
        }
    });
}
