use ambient_api::{
    animation::{AnimationPlayer, AnimationRetargeting, BlendNode, PlayClipFromUrlNode},
    components::core::player::player,
    prelude::*,
};

use crate::components as c;
pub fn register_anim() {
    spawn_query(player()).bind(move |info| {
        for (id, ()) in info {
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
                asset::url("assets/anim/Walk Forward Right.fbx/animations/mixamo.com.anim")
                    .unwrap(),
            );

            let walk_bk_lt = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Walk Backward Left.fbx/animations/mixamo.com.anim")
                    .unwrap(),
            );

            let walk_bk_rt = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Walk Backward Right.fbx/animations/mixamo.com.anim")
                    .unwrap(),
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
                asset::url("assets/anim/Run Backward Right.fbx/animations/mixamo.com.anim")
                    .unwrap(),
            );

            // let run = PlayClipFromUrlNode::new(
            //     asset::url("assets/anim/Rifle Run.fbx/animations/mixamo.com.anim").unwrap(),
            // );
            let jump = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Rifle Jump.fbx/animations/mixamo.com.anim").unwrap(),
            );

            // // Looping is buggy
            jump.looping(false);

            // let hit = PlayClipFromUrlNode::new(
            //     asset::url("assets/anim/Rifle Hit Reaction.fbx/animations/mixamo.com.anim")
            //         .unwrap(),
            // );

            // let death = PlayClipFromUrlNode::new(
            //     asset::url("assets/anim/Rifle Death.fbx/animations/mixamo.com.anim").unwrap(),
            // );

            // // TODO: buggy!!!!!
            // death.looping(false);
            // // death.freeze_at_percentage(100.0);
            // let fire = PlayClipFromUrlNode::new(
            //     asset::url("assets/anim/Firing Rifle.fbx/animations/mixamo.com.anim").unwrap(),
            // );
            // fire.set_retargeting(AnimationRetargeting::Skeleton {
            //     model_url: "assets/model/X Bot.fbx".to_string(),
            // });
            let idle = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Rifle Aiming Idle.fbx/animations/mixamo.com.anim").unwrap(),
            );
            let fd = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Rifle Walking Forward.fbx/animations/mixamo.com.anim")
                    .unwrap(),
            );

            let bk = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Rifle Walking Backward.fbx/animations/mixamo.com.anim")
                    .unwrap(),
            );

            let lt = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Rifle Walk Left.fbx/animations/mixamo.com.anim").unwrap(),
            );

            let rt = PlayClipFromUrlNode::new(
                asset::url("assets/anim/Rifle Walk Right.fbx/animations/mixamo.com.anim").unwrap(),
            );

            let fd_lt = BlendNode::new(&fd, &lt, 0.5);
            let fd_rt = BlendNode::new(&fd, &rt, 0.5);
            let bk_lt = BlendNode::new(&bk, &lt, 0.5);
            let bk_rt = BlendNode::new(&bk, &rt, 0.5);
            let idle_fd = BlendNode::new(&idle, &fd, 0.5);
            let idle_bk = BlendNode::new(&idle, &bk, 0.5);
            let idle_lt = BlendNode::new(&idle, &lt, 0.5);
            let idle_rt = BlendNode::new(&idle, &rt, 0.5);
            let idle_fd_lt = BlendNode::new(&idle, &fd_lt, 0.5);
            let idle_fd_rt = BlendNode::new(&idle, &fd_rt, 0.5);
            let idle_bk_lt = BlendNode::new(&idle, &bk_lt, 0.5);
            let idle_bk_rt = BlendNode::new(&idle, &bk_rt, 0.5);

            let walk_fd_player = AnimationPlayer::new(&walk_fd);
            let walk_bk_player = AnimationPlayer::new(&walk_bk);
            let walk_rt_player = AnimationPlayer::new(&walk_rt);
            let walk_lt_player = AnimationPlayer::new(&walk_lt);
            let walk_fd_lt_player = AnimationPlayer::new(&walk_fd_lt);
            let walk_fd_rt_player = AnimationPlayer::new(&walk_fd_rt);
            let walk_bk_lt_player = AnimationPlayer::new(&walk_bk_lt);
            let walk_bk_rt_player = AnimationPlayer::new(&walk_bk_rt);

            let run_fd_player = AnimationPlayer::new(&run_fd);
            let run_bk_player = AnimationPlayer::new(&run_bk);
            let run_rt_player = AnimationPlayer::new(&run_rt);
            let run_lt_player = AnimationPlayer::new(&run_lt);
            let run_fd_lt_player = AnimationPlayer::new(&run_fd_lt);
            let run_fd_rt_player = AnimationPlayer::new(&run_fd_rt);
            let run_bk_lt_player = AnimationPlayer::new(&run_bk_lt);
            let run_bk_rt_player = AnimationPlayer::new(&run_bk_rt);

            // let run_player = AnimationPlayer::new(&run);
            let jump_player = AnimationPlayer::new(&jump);
            // let hit_player = AnimationPlayer::new(&hit);
            // let death_player = AnimationPlayer::new(&death);
            // let fire_player = AnimationPlayer::new(&fire);
            let fd_lt_player = AnimationPlayer::new(&fd_lt);
            let fd_rt_player = AnimationPlayer::new(&fd_rt);
            let bk_lt_player = AnimationPlayer::new(&bk_lt);
            let bk_rt_player = AnimationPlayer::new(&bk_rt);
            let idle_fd_player = AnimationPlayer::new(&idle_fd);
            let idle_bk_player = AnimationPlayer::new(&idle_bk);
            let idle_lt_player = AnimationPlayer::new(&idle_lt);
            let idle_rt_player = AnimationPlayer::new(&idle_rt);
            let idle_fd_lt_player = AnimationPlayer::new(&idle_fd_lt);
            let idle_fd_rt_player = AnimationPlayer::new(&idle_fd_rt);
            let idle_bk_lt_player = AnimationPlayer::new(&idle_bk_lt);
            let idle_bk_rt_player = AnimationPlayer::new(&idle_bk_rt);

            entity::add_components(
                id,
                Entity::new()
                    .with(c::walk_fd(), vec![walk_fd.0 .0, walk_fd_player.0])
                    .with(c::walk_bk(), vec![walk_bk.0 .0, walk_bk_player.0])
                    .with(c::walk_rt(), vec![walk_rt.0 .0, walk_rt_player.0])
                    .with(c::walk_lt(), vec![walk_lt.0 .0, walk_lt_player.0])
                    .with(c::walk_fd_rt(), vec![walk_fd_rt.0 .0, walk_fd_rt_player.0])
                    .with(c::walk_fd_lt(), vec![walk_fd_lt.0 .0, walk_fd_lt_player.0])
                    .with(c::walk_bk_rt(), vec![walk_bk_rt.0 .0, walk_bk_rt_player.0])
                    .with(c::walk_bk_lt(), vec![walk_bk_lt.0 .0, walk_bk_lt_player.0])
                    .with(c::run_fd(), vec![run_fd.0 .0, run_fd_player.0])
                    .with(c::run_bk(), vec![run_bk.0 .0, run_bk_player.0])
                    .with(c::run_rt(), vec![run_rt.0 .0, run_rt_player.0])
                    .with(c::run_lt(), vec![run_lt.0 .0, run_lt_player.0])
                    .with(c::run_fd_rt(), vec![run_fd_rt.0 .0, run_fd_rt_player.0])
                    .with(c::run_fd_lt(), vec![run_fd_lt.0 .0, run_fd_lt_player.0])
                    .with(c::run_bk_rt(), vec![run_bk_rt.0 .0, run_bk_rt_player.0])
                    .with(c::run_bk_lt(), vec![run_bk_lt.0 .0, run_bk_lt_player.0])
                    // .with(c::run(), vec![run.0 .0, run_player.0])
                    .with(c::jump(), vec![jump.0 .0, jump_player.0])
                    // .with(c::hit(), vec![hit.0 .0, hit_player.0])
                    // .with(c::death(), vec![death.0 .0, death_player.0])
                    // .with(c::fire(), vec![fire.0 .0, fire_player.0])
                    .with(c::fd_lt(), vec![fd_lt.0 .0, fd_lt_player.0])
                    .with(c::fd_rt(), vec![fd_rt.0 .0, fd_rt_player.0])
                    .with(c::bk_lt(), vec![bk_lt.0 .0, bk_lt_player.0])
                    .with(c::bk_rt(), vec![bk_rt.0 .0, bk_rt_player.0])
                    .with(c::idle_fd(), vec![idle_fd.0 .0, idle_fd_player.0])
                    .with(c::idle_bk(), vec![idle_bk.0 .0, idle_bk_player.0])
                    .with(c::idle_lt(), vec![idle_lt.0 .0, idle_lt_player.0])
                    .with(c::idle_rt(), vec![idle_rt.0 .0, idle_rt_player.0])
                    .with(c::idle_fd_lt(), vec![idle_fd_lt.0 .0, idle_fd_lt_player.0])
                    .with(c::idle_fd_rt(), vec![idle_fd_rt.0 .0, idle_fd_rt_player.0])
                    .with(c::idle_bk_lt(), vec![idle_bk_lt.0 .0, idle_bk_lt_player.0])
                    .with(c::idle_bk_rt(), vec![idle_bk_rt.0 .0, idle_bk_rt_player.0]),
            );
        }
    });
}
