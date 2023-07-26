use ambient_api::{
    animation::{AnimationPlayer, AnimationRetargeting, BlendNode, PlayClipFromUrlNode},
    components::core::{
        animation::{apply_animation_player, blend},
        player::player,
    },
    entity::spawn,
    prelude::*,
};

mod anim;

#[main]
pub fn main() {
    anim::register_anim();
    spawn_query((player(), components::player_model_ref())).bind(|v| {
        for (id, (_, model)) in v {
            let jump = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Rifle Jump.fbx/animations/mixamo.com.anim").unwrap(),
            );
            jump.looping(false);
            let jump_player = AnimationPlayer::new(&jump);

            entity::add_component(model, apply_animation_player(), jump_player.0);
            entity::add_component(id, components::player_jumping(), false);
        }
    });

    query((
        player(),
        components::player_model_ref(),
        components::player_direction(),
        components::player_shooting_status(),
        components::player_vspeed(),
        components::player_running(),
        components::player_jumping(),
    ))
    .each_frame(move |results| {
        for (player_id, (_, model, dir, is_shooting, vspeed, is_running, is_jumping)) in results {
            if is_jumping {
                let jump = PlayClipFromUrlNode::new(
                    asset::url("assets/anim/Rifle Jump.fbx/animations/mixamo.com.anim").unwrap(),
                );
                jump.looping(false);
                let anim_player =
                    entity::get_component(model, apply_animation_player()).unwrap_or_default();
                let p = AnimationPlayer(anim_player);
                p.play(jump);
                entity::add_component(player_id, components::player_jumping(), false);
                continue;
            }

            // if is_shooting {
            //     let shoot = PlayClipFromUrlNode::new(
            //         asset::url("assets/anim/Rifle Firing.fbx/animations/mixamo.com.anim").unwrap(),
            //     );
            //     shoot.looping(false);
            //     play_clip_on_model(model, shoot);
            //     continue;
            // }
            let fd = dir.y == -1.0;
            let bk = dir.y == 1.0;
            let lt = dir.x == -1.0;
            let rt = dir.x == 1.0;

            if is_running {
                if fd && !lt && !rt {
                    apply_animation(player_id, components::run_fd());
                } else if bk && !lt && !rt {
                    apply_animation(player_id, components::run_bk());
                } else if lt && !fd && !bk {
                    apply_animation(player_id, components::run_lt());
                } else if rt && !fd && !bk {
                    apply_animation(player_id, components::run_rt());
                } else if fd && lt {
                    apply_animation(player_id, components::run_fd_lt());
                } else if fd && rt {
                    apply_animation(player_id, components::run_fd_rt());
                } else if bk && lt {
                    apply_animation(player_id, components::run_bk_lt());
                } else if bk && rt {
                    apply_animation(player_id, components::run_bk_rt());
                } else {
                    // TODO: there is a bug on multiple animations playing at the same time
                    // I cannot use this commented line
                    // there is a "hijack" bug on the animation player
                    // have to create anim for each player
                    apply_anim_blend(player_id, components::idle_fd(), 0.0);
                    // apply_anim(player_id, components::idle_fd_lt(), 0.0);
                }
                continue;
            } else {
                if fd && !lt && !rt {
                    apply_animation(player_id, components::walk_fd());
                } else if bk && !lt && !rt {
                    apply_animation(player_id, components::walk_bk());
                } else if lt && !fd && !bk {
                    apply_animation(player_id, components::walk_lt());
                } else if rt && !fd && !bk {
                    apply_animation(player_id, components::walk_rt());
                } else if fd && lt {
                    apply_animation(player_id, components::walk_fd_lt());
                } else if fd && rt {
                    apply_animation(player_id, components::walk_fd_rt());
                } else if bk && lt {
                    apply_animation(player_id, components::walk_bk_lt());
                } else if bk && rt {
                    apply_animation(player_id, components::walk_bk_rt());
                } else {
                    // TODO: there is a bug on multiple animations playing at the same time
                    // I cannot use this commented line
                    // there is a "hijack" bug on the animation player
                    // have to create anim for each player
                    apply_anim_blend(player_id, components::idle_fd(), 0.0);
                    // apply_anim(player_id, components::idle_fd_lt(), 0.0);
                }
                continue;
            }

            // let fd = dir.y == -1.0;
            // let bk = dir.y == 1.0;
            // let lt = dir.x == -1.0;
            // let rt = dir.x == 1.0;

            // if fd && !lt && !rt {
            //     play_clip_on_model(
            //         model,
            //         PlayClipFromUrlNode::new(
            //             asset::url("assets/anim/Walk Forward.fbx/animations/mixamo.com.anim")
            //                 .unwrap(),
            //         ),
            //     );
            // } else if bk && !lt && !rt {
            //     play_clip_on_model(
            //         model,
            //         PlayClipFromUrlNode::new(
            //             asset::url("assets/anim/Walk Backward.fbx/animations/mixamo.com.anim")
            //                 .unwrap(),
            //         ),
            //     );
            // } else if lt && !fd && !bk {
            //     play_clip_on_model(
            //         model,
            //         PlayClipFromUrlNode::new(
            //             asset::url("assets/anim/Walk Left.fbx/animations/mixamo.com.anim").unwrap(),
            //         ),
            //     );
            // } else if rt && !fd && !bk {
            //     play_clip_on_model(
            //         model,
            //         PlayClipFromUrlNode::new(
            //             asset::url("assets/anim/Walk Right.fbx/animations/mixamo.com.anim")
            //                 .unwrap(),
            //         ),
            //     );
            // } else if fd && lt {
            //     play_clip_on_model(
            //         model,
            //         PlayClipFromUrlNode::new(
            //             asset::url("assets/anim/Walk Forward Left.fbx/animations/mixamo.com.anim")
            //                 .unwrap(),
            //         ),
            //     );
            // } else if fd && rt {
            //     play_clip_on_model(
            //         model,
            //         PlayClipFromUrlNode::new(
            //             asset::url("assets/anim/Walk Forward Right.fbx/animations/mixamo.com.anim")
            //                 .unwrap(),
            //         ),
            //     );
            // } else if bk && lt {
            //     play_clip_on_model(
            //         model,
            //         PlayClipFromUrlNode::new(
            //             asset::url("assets/anim/Walk Backward Left.fbx/animations/mixamo.com.anim")
            //                 .unwrap(),
            //         ),
            //     );
            // } else if bk && rt {
            //     play_clip_on_model(
            //         model,
            //         PlayClipFromUrlNode::new(
            //             asset::url(
            //                 "assets/anim/Walk Backward Right.fbx/animations/mixamo.com.anim",
            //             )
            //             .unwrap(),
            //         ),
            //     );
            // } else {
            //     play_clip_on_model(
            //         model,
            //         PlayClipFromUrlNode::new(
            //             asset::url("assets/anim/Rifle Aiming Idle.fbx/animations/mixamo.com.anim")
            //                 .unwrap(),
            //         ),
            //     );
            // }
        }
    });

    change_query((
        player(),
        components::player_health(),
        components::player_model_ref(),
    ))
    .track_change(components::player_health())
    .bind(|v| {
        // play hit animation
        for (_id, (_, health, model)) in v {
            if health <= 0 {
                let death = PlayClipFromUrlNode::new(
                    asset::url("assets/anim/Rifle Death.fbx/animations/mixamo.com.anim").unwrap(),
                );
                death.looping(false);
                play_clip_on_model(model, death);
            } else if health < 100 {
                let hit = PlayClipFromUrlNode::new(
                    asset::url("assets/anim/Rifle Hit Reaction.fbx/animations/mixamo.com.anim")
                        .unwrap(),
                );
                hit.looping(false);
                play_clip_on_model(model, hit);
            }
        }
    });
}

fn play_clip_on_model(model: EntityId, clip: PlayClipFromUrlNode) {
    let anim_player = entity::get_component(model, apply_animation_player()).unwrap_or_default();
    let p = AnimationPlayer(anim_player);
    p.play(clip);
}

pub fn apply_animation(player_id: EntityId, comp: Component<Vec<EntityId>>) {
    let model = entity::get_component(player_id, components::player_model_ref());
    if model.is_none() {
        return;
    }
    let model = model.unwrap();
    let anim_player = entity::get_component(player_id, comp);
    if anim_player.is_none() {
        return;
    }
    let anim_player = anim_player.unwrap();
    entity::add_component(model, apply_animation_player(), anim_player[1]);
}

pub fn apply_anim_blend(player_id: EntityId, comp: Component<Vec<EntityId>>, blend_value: f32) {
    let model = entity::get_component(player_id, components::player_model_ref());
    if model.is_none() {
        return;
    }
    let model = model.unwrap();
    let blend_player = entity::get_component(player_id, comp);
    if blend_player.is_none() {
        return;
    }
    let blend_player = blend_player.unwrap();
    entity::set_component(blend_player[0], blend(), blend_value);
    entity::add_component(model, apply_animation_player(), blend_player[1]);
}
