use ambient_api::{
    core::{
        animation::components::{apply_animation_player, blend},
        player::components::player,
    },
    prelude::*,
};

use afps::{
    afps_fpsanim::components::{
        bk_lt, bk_rt, death, fd_lt, fd_rt, fire, hit, idle_bk, idle_fd, idle_lt, idle_rt, jump,
        run, run_bk, run_bk_lt, run_bk_rt, run_fd, run_fd_lt, run_fd_rt, run_lt, run_rt,
    },
    afps_fpsmodel::components::player_model_ref,
    afps_fpsmovement::components::{
        player_direction, player_running, player_shooting_status, player_vspeed,
    },
    afps_fpsrule::components::{hit_freeze, player_health},
};

mod anim;

#[main]
pub fn main() {
    anim::register_anim();
    query((
        player(),
        player_model_ref(),
        player_direction(),
        player_shooting_status(),
        player_vspeed(),
        player_running(),
        jump(),
        fire(),
        run(),
    ))
    .each_frame(|results| {
        for (
            player_id,
            (_, model, dir, is_shooting, vspeed, is_running, jump_anim, fire_anim, run_anim),
        ) in results
        {
            if vspeed.abs() > 0.07 {
                entity::add_component(model, apply_animation_player(), jump_anim[1]);
                continue;
            }

            // this is added later with the rules
            // the main takeaway is that each mod is not always self contained
            // for example, the hit_freeze is added in a mod called `rule`
            // but for its anim, we should add it here
            if let Some(freeze) = entity::get_component(player_id, hit_freeze()) {
                if freeze > 0 {
                    entity::set_component(player_id, hit_freeze(), freeze - 1);
                    continue;
                }
            };

            if is_shooting {
                entity::add_component(model, apply_animation_player(), fire_anim[1]);
                continue;
            };

            if is_running {
                let fd = dir.y == -1.0;
                let bk = dir.y == 1.0;
                let lt = dir.x == -1.0;
                let rt = dir.x == 1.0;

                if fd && !lt && !rt {
                    apply_animation(player_id, run_fd());
                } else if bk && !lt && !rt {
                    apply_animation(player_id, run_bk());
                } else if lt && !fd && !bk {
                    apply_animation(player_id, run_lt());
                } else if rt && !fd && !bk {
                    apply_animation(player_id, run_rt());
                } else if fd && lt {
                    apply_animation(player_id, run_fd_lt());
                } else if fd && rt {
                    apply_animation(player_id, run_fd_rt());
                } else if bk && lt {
                    apply_animation(player_id, run_bk_lt());
                } else if bk && rt {
                    apply_animation(player_id, run_bk_rt());
                } else {
                    // TODO: there is a bug on multiple animations playing at the same time
                    // I cannot use this commented line
                    // there is a "hijack" bug on the animation player
                    // have to create anim for each player
                    apply_anim(player_id, idle_fd(), 0.0);
                    // apply_anim(player_id, idle_fd_lt(), 0.0);
                }
                continue;
            };

            let fd = dir.y == -1.0;
            let bk = dir.y == 1.0;
            let lt = dir.x == -1.0;
            let rt = dir.x == 1.0;

            if fd && !lt && !rt {
                apply_anim(player_id, idle_fd(), 1.0);
            } else if bk && !lt && !rt {
                apply_anim(player_id, idle_bk(), 1.0);
            } else if lt && !fd && !bk {
                apply_anim(player_id, idle_lt(), 1.0);
            } else if rt && !fd && !bk {
                apply_anim(player_id, idle_rt(), 1.0);
            } else if fd && lt {
                apply_anim(player_id, fd_lt(), 0.5);
            } else if fd && rt {
                apply_anim(player_id, fd_rt(), 0.5);
            } else if bk && lt {
                apply_anim(player_id, bk_lt(), 0.5);
            } else if bk && rt {
                apply_anim(player_id, bk_rt(), 0.5);
            } else {
                // TODO: there is a bug on multiple animations playing at the same time
                // I cannot use this commented line
                // there is a "hijack" bug on the animation player
                // have to create anim for each player
                apply_anim(player_id, idle_fd(), 0.0);
                // apply_anim(player_id, idle_fd_lt(), 0.0);
            }
        }
    });

    // this is also added later with the rule mod
    // but for its anim, we should add it here
    // play `hit reaction` or `death` animation
    change_query((
        player(),
        player_health(),
        player_model_ref(),
        death(),
        hit(),
    ))
    .track_change(player_health())
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
    let model = entity::get_component(player_id, player_model_ref());
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

pub fn apply_animation(player_id: EntityId, comp: Component<Vec<EntityId>>) {
    let model = entity::get_component(player_id, player_model_ref());
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
