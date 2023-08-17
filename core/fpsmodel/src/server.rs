use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::{aspect_ratio_from_window, fovy},
            concepts::make_perspective_infinite_reverse_camera,
        },
        ecs::components::{children, parent},
        physics::components::{
            character_controller_height, character_controller_radius, physics_controlled,
        },
        player::components::{is_player, user_id},
        prefab::components::prefab_from_url,
        transform::{
            components::{local_to_parent, local_to_world, rotation, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use packages::{
    afps_fpsmodel::assets,
    afps_schema::components::{player_cam_ref, player_model_ref, player_name, player_zoomed},
};

#[main]
pub async fn main() {
    query((is_player(), player_zoomed(), player_cam_ref())).each_frame(|v| {
        for (_, ((), zoomed, cam_ref)) in v {
            entity::set_component(cam_ref, fovy(), if zoomed { 0.3 } else { 1.0 })
        }
    });
    spawn_query((is_player(), user_id())).bind(move |players| {
        for (id, (_, uid)) in players {
            run_async(async move {
                entity::wait_for_component(id, player_name()).await;

                // refer to the first person example in Ambient repo
                let cam = Entity::new()
                    .with_merge(make_perspective_infinite_reverse_camera())
                    .with(aspect_ratio_from_window(), EntityId::resources())
                    .with(main_scene(), ())
                    .with(parent(), id)
                    .with(user_id(), uid)
                    // this is FPS
                    // .with(translation(), vec3(0.0, 0.2, 2.0))
                    // third person
                    .with(translation(), vec3(0.0, 4.0, 3.0))
                    .with(local_to_parent(), Default::default())
                    // .with(local_to_world(), Default::default())
                    .with(
                        rotation(),
                        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    )
                    .spawn();
                let model = Entity::new()
                    .with_merge(make_transformable())
                    .with(prefab_from_url(), assets::url("Y Bot.fbx"))
                    .with(rotation(), Quat::from_rotation_z(-std::f32::consts::PI))
                    .with(local_to_parent(), Default::default())
                    .with(parent(), id)
                    .spawn();
                entity::add_components(
                    id,
                    Entity::new()
                        .with_merge(make_transformable())
                        // with the following three comp, you can move the player
                        // with physics::move_character
                        .with(character_controller_height(), 2.0)
                        .with(character_controller_radius(), 0.3)
                        .with(physics_controlled(), ())
                        // adjust the initial position
                        .with(local_to_world(), Default::default())
                        .with(
                            translation(),
                            vec3(random::<f32>() * 20., random::<f32>() * 20., 2.0),
                        )
                        .with(children(), vec![model, cam])
                        .with(player_cam_ref(), cam)
                        .with(player_model_ref(), model),
                );
            });
        }
    });
}
