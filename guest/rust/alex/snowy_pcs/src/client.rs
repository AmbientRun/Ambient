use ambient_api::{
    core::{model::components::model_from_url, player::components::is_player},
    prelude::*,
};
use packages::character_animation::components::basic_character_animations;
use packages::fps_controller::components::player_intermediate_rotation;
use packages::this::components::{dead_age, pc_type_id};

#[main]
pub fn main() {
    let chicken_anims = make_chicken_anims();

    spawn_query(pc_type_id())
        .requires(is_player())
        .bind(move |plrs| {
            for (plr, plr_type_id) in plrs {
                entity::add_components(plr, chicken_anims.clone());
                entity::add_component(plr, basic_character_animations(), plr);
                update_fps_model(plr, plr_type_id);
            }
        });

    change_query(pc_type_id())
        .track_change(pc_type_id())
        .bind(|plrs| {
            for (plr, plr_type_id) in plrs {
                update_fps_model(plr, plr_type_id);
            }
        });

    ambient_api::core::messages::Frame::subscribe(|_| {
        let (delta, _input) = input::get_delta();
        if delta.keys.contains(&KeyCode::M) {
            packages::this::messages::SwitchType {}.send_server_reliable();
        }

        let player_id = player::get_local();
        if entity::has_component(player_id, dead_age()) {
            // if player is dead, reset camera
            entity::add_component(player_id, player_intermediate_rotation(), Vec2::ZERO);
        }
    });
}

fn update_fps_model(fps_entity: EntityId, fps_entity_type_id: u32) {
    entity::add_components(
        fps_entity,
        Entity::new()
            .with(model_from_url(), type_id_to_model_path(fps_entity_type_id))
            .with(idle(), type_id_to_idle_path(fps_entity_type_id))
            .with(jump(), type_id_to_jump_path(fps_entity_type_id)),
    );
}
fn type_id_to_model_path(type_id: u32) -> String {
    match type_id % 2 {
        // 1 => packages::this::assets::url("model-chkn/muscle batch bear.fbx"), // does not work
        _ => packages::this::assets::url("model-chkn/Chicken Rebuilt Mixamo T-Pose.fbx"), // this is the one that works!!!
    }
}
fn type_id_to_idle_path(type_id: u32) -> String {
    match type_id % 4 {
        3 => anim_chkn("Old Man Idle"),
        2 => anim_chkn("Happy Idle"),
        1 => anim_chkn("Shoulder Rubbing Idle"),
        _ => anim_chkn("Offensive Idle"),
    }
}

fn type_id_to_jump_path(type_id: u32) -> String {
    match type_id {
        _ => anim_chkn("Joyful Jump"), // this is the one that works!!!
    }
}

const CHKN_PREFIX: &str = "anim-chkn-jog/";
use packages::character_animation::components::*;
fn anim_chkn(name: &str) -> String {
    anim_url((CHKN_PREFIX.to_string() + name).as_mut_str())
}
fn anim_url(name: &str) -> String {
    packages::this::assets::url(&format!("{name}.fbx/animations/mixamo.com.anim"))
}

fn make_chicken_anims() -> Entity {
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
