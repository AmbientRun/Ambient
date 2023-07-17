use ambient_api::{
    components::core::{
        animation::{apply_animation_player, blend},
        player::player,
    },
    prelude::*,
};

mod anim;

#[main]
pub fn main() {
    anim::register_anim();
    query((
        player(),
        components::player_model_ref(),
        components::player_direction(),
        components::player_shooting_status(),
        components::player_vspeed(),
        components::jump(),
        components::fire(),
    ))
    .each_frame(|results| {
        for (player_id, (_, model, dir, is_shooting, vspeed, jump_anim, fire_anim)) in results {
            if vspeed.abs() > 0.07 {
                entity::add_component(model, apply_animation_player(), jump_anim[1]);
                continue;
            }

            // this is added later with the rules
            // the main takeaway is that each mod is not always self contained
            // for example, the hit_freeze is added in a mod called `rule`
            // but for its anim, we should add it here
            if let Some(freeze) = entity::get_component(player_id, components::hit_freeze()) {
                if freeze > 0 {
                    entity::set_component(player_id, components::hit_freeze(), freeze - 1);
                    continue;
                }
            };

            if is_shooting {
                entity::add_component(model, apply_animation_player(), fire_anim[1]);
                continue;
            };
            let fd = dir.y == -1.0;
            let bk = dir.y == 1.0;
            let lt = dir.x == -1.0;
            let rt = dir.x == 1.0;

            if fd && !lt && !rt {
                apply_anim(player_id, components::idle_fd(), 1.0);
            } else if bk && !lt && !rt {
                apply_anim(player_id, components::idle_bk(), 1.0);
            } else if lt && !fd && !bk {
                apply_anim(player_id, components::idle_lt(), 1.0);
            } else if rt && !fd && !bk {
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
                // TODO: there is a bug on multiple animations playing at the same time
                // I cannot use this commented line
                // there is a "hijack" bug on the animation player
                // have to create anim for each player
                apply_anim(player_id, components::idle_fd(), 0.0);
                // apply_anim(player_id, components::idle_fd_lt(), 0.0);
            }
        }
    });

    // this is also added later with the rule mod
    // but for its anim, we should add it here
    // play `hit reaction` or `death` animation
    change_query((
        player(),
        components::player_health(),
        components::player_model_ref(),
        components::death(),
        components::hit(),
    ))
    .track_change(components::player_health())
    .bind(|v| {
        // play hit animation
        for (_id, (_, health, model, death_anim, hit_anim)) in v {
            if health <= 0 {
                entity::add_component(model, apply_animation_player(), death_anim[1]);
            } else if health < 100 {
                entity::add_component(model, apply_animation_player(), hit_anim[1]);
            }
        }
    });
}

pub fn apply_anim(player_id: EntityId, comp: Component<Vec<EntityId>>, blend_value: f32) {
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
