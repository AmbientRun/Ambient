use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::{aspect_ratio_from_window, fovy},
            concepts::make_PerspectiveInfiniteReverseCamera,
        },
        ecs::components::{children, parent},
        physics::concepts::make_CharacterController,
        player::components::{is_player, user_id},
        prefab::components::prefab_from_url,
        transform::{
            components::{local_to_parent, local_to_world, rotation, translation},
            concepts::make_Transformable,
        },
    },
    entity::set_component,
    prelude::*,
};

use packages::{
    afps_schema::components::{player_cam_ref, player_model_ref, player_name, player_zoomed},
    basic_character_animation::components::basic_character_animations,
    this::assets,
    unit_schema::components::head_ref,
};
use std::f32::consts::PI;

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
                if entity::wait_for_component(id, player_name())
                    .await
                    .is_none()
                {
                    // entity deleted
                    return;
                }

                let cam = Entity::new()
                    .with_merge(make_PerspectiveInfiniteReverseCamera())
                    .with(aspect_ratio_from_window(), EntityId::resources())
                    .with(main_scene(), ())
                    .with(translation(), -Vec3::Z * 4.)
                    .with(parent(), EntityId::null())
                    .with(local_to_parent(), Default::default())
                    .with(user_id(), uid)
                    .spawn();

                let head = Entity::new()
                    .with_merge(make_transformable())
                    .with(local_to_parent(), Default::default())
                    .with(parent(), id)
                    .with(translation(), Vec3::Z * 2.)
                    .with(
                        rotation(),
                        Quat::from_rotation_z(PI / 2.) * Quat::from_rotation_x(PI / 2.),
                    )
                    .with(children(), vec![cam])
                    .spawn();
                set_component(cam, parent(), head);

                let model = Entity::new()
                    .with_merge(make_Transformable())
                    .with(prefab_from_url(), assets::url("Y Bot.fbx"))
                    .with(
                        rotation(),
                        Quat::from_rotation_z(-std::f32::consts::PI / 2.),
                    )
                    .with(local_to_parent(), Default::default())
                    .with(parent(), id)
                    .spawn();
                entity::add_components(
                    id,
                    Entity::new()
                        .with_merge(make_Transformable())
                        .with_merge(make_CharacterController())
                        .with(basic_character_animations(), model)
                        // adjust the initial position
                        .with(local_to_world(), Default::default())
                        .with(
                            translation(),
                            vec3(random::<f32>() * 20., random::<f32>() * 20., 2.0),
                        )
                        .with(children(), vec![model, head])
                        .with(player_cam_ref(), cam)
                        .with(player_model_ref(), model)
                        .with(head_ref(), head),
                );
            });
        }
    });
}
