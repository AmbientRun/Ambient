use ambient_api::{
    core::{
        camera::components::fovy, model::components::model_from_url, player::components::is_player,
        transform::components::translation,
    },
    prelude::*,
};

use packages::{
    afps_schema::components::{player_cam_ref, player_name, player_zoomed},
    base_assets,
    character_animation::components::basic_character_animations,
    fps_controller::components::use_fps_controller,
};

#[main]
pub async fn main() {
    query((is_player(), player_zoomed(), player_cam_ref())).each_frame(|v| {
        for (_, ((), zoomed, cam_ref)) in v {
            entity::set_component(cam_ref, fovy(), if zoomed { 0.3 } else { 1.0 })
        }
    });
    spawn_query(is_player()).bind(move |players| {
        for (id, _) in players {
            run_async(async move {
                if entity::wait_for_component(id, player_name())
                    .await
                    .is_none()
                {
                    // entity deleted
                    return;
                }

                entity::add_components(
                    id,
                    Entity::new()
                        .with(use_fps_controller(), ())
                        .with(model_from_url(), base_assets::assets::url("Y Bot.fbx"))
                        .with(basic_character_animations(), id)
                        .with(
                            translation(),
                            vec3(random::<f32>() * 20., random::<f32>() * 20., 2.0),
                        ),
                );
            });
        }
    });
}
