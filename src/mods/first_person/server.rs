use ambient_api::{
    animation::{AnimationPlayer, BlendNode, PlayClipFromUrlNode},
    components::core::{
        animation::{animation_player, apply_animation_player},
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

use components::{player_head_ref, player_movement_direction, player_pitch, player_yaw};
use std::f32::consts::{PI, TAU};

#[main]
pub fn main() {
    let idle = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Rifle Aiming Idle.fbx/animations/mixamo.com.anim").unwrap(),
    );
    let walk = PlayClipFromUrlNode::new(
        asset::url("assets/anim/Rifle Walking Forward.fbx/animations/mixamo.com.anim").unwrap(),
    );
    // let attack = PlayClipFromUrlNode::new(
    //     asset::url("assets/anim/Standing Torch Melee Attack 01.fbx/animations/mixamo.com.anim")
    //         .unwrap(),
    // );
    let idle_player = AnimationPlayer::new(&idle);
    let walk_player = AnimationPlayer::new(&walk);
    // let attack_player = AnimationPlayer::new(&attack);
    // let cam = Entity::new()
    //     .with_merge(make_perspective_infinite_reverse_camera())
    //     .with(aspect_ratio_from_window(), EntityId::resources())
    //     .with_default(main_scene())
    //     .with(translation(), vec3(10.0, 0.0, 10.0) * 1.0)
    //     .with(lookat_target(), vec3(0., 0., 0.))
    //     .spawn();

    spawn_query(player()).bind(move |players| {
        for (id, _) in players {
            let cam = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with(aspect_ratio_from_window(), EntityId::resources())
                .with_default(main_scene())
                // .with(user_id(), uid.clone())
                .with(translation(), vec3(-0.0, 5.0, 3.0))
                .with(parent(), id)
                .with_default(local_to_parent())
                // .with_default(local_to_world())
                .with(rotation(), Quat::from_rotation_x(PI / 2.0))
                .spawn();
            let model = Entity::new()
                .with_merge(make_transformable())
                .with(
                    prefab_from_url(),
                    asset::url("assets/model/Y Bot.fbx").unwrap(),
                )
                .with(rotation(), Quat::from_rotation_z(-3.14159265359))
                .with_default(local_to_parent())
                .with(parent(), id)
                .spawn();
            entity::add_component(model, apply_animation_player(), idle_player.0);
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_transformable())
                    .with(character_controller_height(), 2.0)
                    .with(character_controller_radius(), 0.5)
                    .with_default(physics_controlled())
                    .with(children(), vec![model, cam])
                    .with(player_head_ref(), cam)
                    .with(components::model_ref(), model)
                    .with(player_pitch(), 0.0)
                    .with(player_yaw(), 0.0)
                    .with(translation(), vec3(0., 0., 5.))
                    .with_default(local_to_world()),
            );
        }
    });

    messages::Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        entity::add_component(
            player_id,
            components::player_movement_direction(),
            msg.direction,
        );

        if msg.direction != Vec2::ZERO {
            let model = entity::get_component(player_id, components::model_ref()).unwrap();
            entity::add_component(model, apply_animation_player(), walk_player.0);
        }

        let yaw = entity::mutate_component(player_id, components::player_yaw(), |yaw| {
            *yaw = (*yaw + msg.mouse_delta.x * 0.01) % TAU;
        })
        .unwrap_or_default();
        let pitch = entity::mutate_component(player_id, player_pitch(), |pitch| {
            *pitch = (*pitch + msg.mouse_delta.y * 0.01).clamp(-PI / 3., PI / 3.);
        })
        .unwrap_or_default();
        entity::set_component(player_id, rotation(), Quat::from_rotation_z(yaw));
        if let Some(head_id) = entity::get_component(player_id, player_head_ref()) {
            entity::set_component(head_id, rotation(), Quat::from_rotation_x(PI / 2. + pitch));
        }
    });

    messages::Ray::subscribe(move |_source, msg| {
        let result = physics::raycast_first(msg.ray_origin, msg.ray_dir);
        if let Some(hit) = result {
            // println!("{:?}", hit);
            if entity::has_component(hit.entity, components::is_zombie()) && msg.type_action == 0 {
                println!("hit zombie");
                messages::Hit::new(hit.entity).send_client_broadcast_unreliable();
                entity::mutate_component(hit.entity, components::zombie_health(), |x| {
                    if *x <= 0 {
                        return;
                    } else {
                        *x -= 100;
                    }
                });
            }
        }
    });

    query((player(), player_movement_direction(), rotation())).each_frame(move |players| {
        for (player_id, (_, direction, rot)) in players {
            let speed = 0.1;
            let displace = rot * (direction.normalize_or_zero() * speed).extend(-0.1);
            // println!("displace: {:?}", displace);
            let collision = physics::move_character(player_id, displace, 0.01, frametime());
            // println!("collision: {} {} {}", collision.up, collision.down, collision.side);
        }
    });
}
