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
    let jump = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Rifle Jump.fbx/animations/mixamo.com.anim").unwrap(),
    );
    jump.looping(false);
    let jump_player = AnimationPlayer::new(&jump);
    spawn_query((player(), components::player_model_ref())).bind(move |v| {
        for (id, (_, model)) in v {
            entity::add_component(model, apply_animation_player(), jump_player.clone().0);
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
                jump_player.play(jump);
                entity::add_component(player_id, components::player_jumping(), false);
                continue;
            }
        }
    });
}
