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

use components::{player_head_ref, player_movement_direction, player_pitch, player_yaw};
use std::f32::consts::{E, PI, TAU};

const MAX_SPEED: f32 = 0.1;
const SPEED_DELTA: f32 = 0.01;
mod anim;
#[main]
pub fn main() {
    anim::register_anim();
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
            // entity::add_component(model, apply_animation_player(), idle_player.0);
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
                    .with(components::speed(), 0.0)
                    .with(components::running(), false)
                    .with(components::offground(), false)
                    .with(components::player_health(), 100)
                    .with(player_pitch(), 0.0)
                    .with(player_yaw(), 0.0)
                    .with(translation(), vec3(0., 0., 5.))
                    .with_default(local_to_world()),
            );
        }
    });

    messages::Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        let previous_direction =
            entity::get_component(player_id, components::player_movement_direction())
                .unwrap_or_default();

        entity::add_component(
            player_id,
            components::player_movement_direction(),
            msg.direction,
        );

        let speed;
        let fd = msg.direction.y == -1.0;
        let bk = msg.direction.y == 1.0;
        let lt = msg.direction.x == -1.0;
        let rt = msg.direction.x == 1.0;

        let old_fd = previous_direction.y == -1.0;
        let old_bk = previous_direction.y == 1.0;
        let old_lt = previous_direction.x == -1.0;
        let old_rt = previous_direction.x == 1.0;

        let model = entity::get_component(player_id, components::model_ref()).unwrap();
        if msg.direction == Vec2::ZERO {
            // stops, but we need to blend to idle depends on previous direction
            let old_speed = entity::get_component(player_id, components::speed()).unwrap();
            speed = (old_speed - SPEED_DELTA).max(0.0);
            entity::set_component(player_id, components::speed(), speed);
            let bld = speed / MAX_SPEED;
            if old_fd {
                from_move_to_idle(model, bld, components::idle_fd())
            } else if old_bk {
                from_move_to_idle(model, bld, components::idle_bk())
            } else if old_lt {
                from_move_to_idle(model, bld, components::idle_lt())
            } else if old_rt {
                from_move_to_idle(model, bld, components::idle_rt())
            } else if old_fd && old_lt {
                from_move_to_idle(model, bld, components::idle_fd_lt())
            } else if old_fd && old_rt {
                from_move_to_idle(model, bld, components::idle_fd_rt())
            } else if old_bk && old_lt {
                from_move_to_idle(model, bld, components::idle_bk_lt())
            } else if old_bk && old_rt {
                from_move_to_idle(model, bld, components::idle_bk_rt())
            } else {
                from_move_to_idle(model, bld, components::idle_fd())
            }
        } else if fd && !lt && !rt {
            speed = from_idle_to_move(player_id, components::idle_fd());
        } else if bk && !lt && !rt {
            speed = from_idle_to_move(player_id, components::idle_bk());
        } else if lt && !fd && !bk {
            speed = from_idle_to_move(player_id, components::idle_lt());
        } else if rt && !fd && !bk {
            speed = from_idle_to_move(player_id, components::idle_rt());
        } else if fd && lt {
            speed = from_idle_to_move(player_id, components::idle_fd_lt());
        } else if fd && rt {
            speed = from_idle_to_move(player_id, components::idle_fd_rt());
        } else if bk && lt {
            speed = from_idle_to_move(player_id, components::idle_bk_lt());
        } else if bk && rt {
            speed = from_idle_to_move(player_id, components::idle_bk_rt());
        } else {
            speed = from_idle_to_move(player_id, components::idle_fd());
        }
        if speed < 0.0 {
            // this is a dummy function
            return;
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
            } else if entity::has_component(hit.entity, components::player_health())
                && msg.type_action == 0
            {
                println!("hit player");
            }
        }
    });

    query((
        player(),
        player_movement_direction(),
        rotation(),
        components::speed(),
    ))
    .each_frame(move |players| {
        for (player_id, (_, direction, rot, speed)) in players {
            // let speed = 0.1;
            let displace = rot * (direction.normalize_or_zero() * speed).extend(-0.1);
            // println!("displace: {:?}", displace);
            let collision = physics::move_character(player_id, displace, 0.01, frametime());
            // println!("collision: {} {} {}", collision.up, collision.down, collision.side);
        }
    });
}

pub fn from_idle_to_move(player_id: EntityId, comp: Component<Vec<EntityId>>) -> f32 {
    let model = entity::get_component(player_id, components::model_ref()).unwrap();
    let old_speed = entity::get_component(player_id, components::speed()).unwrap();
    let speed = (old_speed + SPEED_DELTA).min(MAX_SPEED);
    entity::set_component(player_id, components::speed(), speed);
    let blend_player = entity::get_component(entity::resources(), comp).unwrap();
    entity::set_component(blend_player[0], blend(), speed / MAX_SPEED);
    entity::add_component(model, apply_animation_player(), blend_player[1]);
    speed
}

pub fn from_move_to_idle(model: EntityId, bld: f32, comp: Component<Vec<EntityId>>) {
    let blend_player = entity::get_component(entity::resources(), comp).unwrap();
    entity::set_component(blend_player[0], blend(), bld);
    entity::add_component(model, apply_animation_player(), blend_player[1]);
}
