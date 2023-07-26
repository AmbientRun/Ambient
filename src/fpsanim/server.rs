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
                play_clip_on_model(model, jump);
                entity::add_component(player_id, components::player_jumping(), false);
                continue;
            }

            if is_shooting {
                let shoot = PlayClipFromUrlNode::new(
                    asset::url("assets/anim/Rifle Firing.fbx/animations/mixamo.com.anim").unwrap(),
                );
                shoot.looping(false);
                play_clip_on_model(model, shoot);
                continue;
            }

            let fd = dir.y == -1.0;
            let bk = dir.y == 1.0;
            let lt = dir.x == -1.0;
            let rt = dir.x == 1.0;

            if fd && !lt && !rt {
                play_clip_on_model(
                    model,
                    PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Walk Forward.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    ),
                );
            } else if bk && !lt && !rt {
                play_clip_on_model(
                    model,
                    PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Walk Backward.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    ),
                );
            } else if lt && !fd && !bk {
                play_clip_on_model(
                    model,
                    PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Walk Left.fbx/animations/mixamo.com.anim").unwrap(),
                    ),
                );
            } else if rt && !fd && !bk {
                play_clip_on_model(
                    model,
                    PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Walk Right.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    ),
                );
            } else if fd && lt {
                play_clip_on_model(
                    model,
                    PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Walk Forward Left.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    ),
                );
            } else if fd && rt {
                play_clip_on_model(
                    model,
                    PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Walk Forward Right.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    ),
                );
            } else if bk && lt {
                play_clip_on_model(
                    model,
                    PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Walk Backward Left.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    ),
                );
            } else if bk && rt {
                play_clip_on_model(
                    model,
                    PlayClipFromUrlNode::new(
                        asset::url(
                            "assets/anim/Walk Backward Right.fbx/animations/mixamo.com.anim",
                        )
                        .unwrap(),
                    ),
                );
            } else {
                play_clip_on_model(
                    model,
                    PlayClipFromUrlNode::new(
                        asset::url("assets/anim/Rifle Aiming Idle.fbx/animations/mixamo.com.anim")
                            .unwrap(),
                    ),
                );
            }
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
