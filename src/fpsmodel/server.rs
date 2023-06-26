#[allow(unused_imports)]
use ambient_api::{
    animation::{AnimationPlayer, BlendNode, PlayClipFromUrlNode},
    components::core::{
        animation::{apply_animation_player, blend},
        app::main_scene,
        camera::aspect_ratio_from_window,
        ecs::{children, parent},
        physics::{
            character_controller_height, character_controller_radius, physics_controlled,
            plane_collider, sphere_collider,
        },
        player::{player, user_id},
        prefab::prefab_from_url,
        primitives::{cube, quad},
        rendering::color,
        transform::{local_to_parent, rotation, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};

#[main]
pub fn main() {
    spawn_query((player(), user_id())).bind(move |players| {
        for (id, (_, uid)) in players {
            let cam = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with(aspect_ratio_from_window(), EntityId::resources())
                .with_default(main_scene())
                .with(parent(), id)
                .with(user_id(), uid)
                // this is FPS
                // .with(translation(), vec3(0.0, 0.2, 2.0))
                // third person
                .with(translation(), vec3(0.0, 4.0, 3.0))
                .with_default(local_to_parent())
                // .with_default(local_to_world())
                .with(
                    rotation(),
                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                )
                .spawn();
            let model = Entity::new()
                .with_merge(make_transformable())
                .with(
                    prefab_from_url(),
                    asset::url("assets/model/Y Bot.fbx").unwrap(),
                )
                .with(rotation(), Quat::from_rotation_z(-std::f32::consts::PI))
                .with_default(local_to_parent())
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
                    .with_default(physics_controlled())
                    // adjust the initial position
                    .with_default(local_to_world())
                    .with(
                        translation(),
                        vec3(random::<f32>() * 20., random::<f32>() * 20., 2.0),
                    )
                    // .with(
                    //     translation(),
                    //     vec3(random::<f32>() * 10.0, random::<f32>() * 10.0, 50.),
                    // )
                    .with(children(), vec![model, cam])
                    .with(components::player_cam_ref(), cam)
                    .with(components::player_model_ref(), model),
            );
        }
    });
}
