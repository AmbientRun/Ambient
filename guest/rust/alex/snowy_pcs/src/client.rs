use ambient_api::{
    core::{
        app::components::name, model::components::model_from_url, player::components::is_player,
        transform::components::rotation,
    },
    prelude::*,
};
use packages::character_animation::components::basic_character_animations;
use packages::fps_controller::components::player_intermediate_rotation;
use packages::this::components::dead_age;

#[main]
pub fn main() {
    let chicken_anims = make_chicken_anims();

    spawn_query(()).requires(is_player()).bind(move |plrs| {
        for (plr, _) in plrs {
            entity::add_components(
                plr,
                chicken_anims
                    .clone()
                    .with(
                        model_from_url(),
                        packages::this::assets::url("model-chkn/Chicken Rebuilt Mixamo T-Pose.fbx"), // this is the one that works!!!
                    )
                    .with(basic_character_animations(), plr),
            );
        }
    });

    ambient_api::core::messages::Frame::subscribe(|_| {
        let player_id = player::get_local();
        if entity::has_component(player_id, dead_age()) {
            // if player is dead, reset camera
            entity::add_component(player_id, player_intermediate_rotation(), Vec2::ZERO);
        }
    });
}

fn make_chicken_anims() -> Entity {
    const CHKN_PREFIX: &str = "anim-chkn-jog/";
    use packages::character_animation::components::*;
    fn anim_chkn(name: &str) -> String {
        anim_url((CHKN_PREFIX.to_string() + name).as_mut_str())
    }
    fn anim_url(name: &str) -> String {
        packages::this::assets::url(&format!("{name}.fbx/animations/mixamo.com.anim"))
    }
    Entity::new()
        .with(death(), anim_chkn("Fallen Idle"))
        .with(idle(), anim_chkn("Offensive Idle"))
        .with(jump(), anim_chkn("Joyful Jump"))
        .with(walk_forward(), anim_chkn("Jog Forward"))
        .with(walk_forward_left(), anim_chkn("Jog Forward Diagonal"))
        .with(walk_forward_right(), anim_chkn("Jog Forward Diagonal (1)"))
        .with(walk_right(), anim_chkn("Strafe"))
        .with(walk_backward(), anim_chkn("Jog Backward"))
        .with(walk_backward_left(), anim_chkn("Jog Backward Diagonal (1)"))
        .with(walk_backward_right(), anim_chkn("Jog Backward Diagonal"))
        .with(walk_left(), anim_chkn("Strafe (1)"))
        .with(run_forward(), anim_chkn("Jog Forward"))
        .with(run_forward_left(), anim_chkn("Jog Forward Diagonal"))
        .with(run_forward_right(), anim_chkn("Jog Forward Diagonal (1)"))
        .with(run_right(), anim_chkn("Strafe"))
        .with(run_backward(), anim_chkn("Jog Backward"))
        .with(run_backward_left(), anim_chkn("Jog Backward Diagonal (1)"))
        .with(run_backward_right(), anim_chkn("Jog Backward Diagonal"))
        .with(run_left(), anim_chkn("Strafe (1)"))
}
