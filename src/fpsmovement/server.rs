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
    spawn_query(player()).bind(|results| {
        for (id, _) in results {
            entity::add_component(id, components::player_yaw(), 0.0);
            entity::add_component(id, components::player_pitch(), 0.0);
            entity::add_component(id, components::player_vspeed(), 0.0);
        }
    });
    messages::Input::subscribe(|source, msg| {
        // receive movement and send this for further processing
        let player_id = source.client_entity_id().unwrap();
        let direction = msg.direction;

        // temporary fix pos for shooting
        if !msg.is_shooting {
            entity::add_component(player_id, components::player_direction(), direction);
        } else {
            entity::add_component(player_id, components::player_direction(), Vec2::ZERO);
        }

        entity::add_component(
            player_id,
            components::player_shooting_status(),
            msg.is_shooting,
        );
        let yaw = entity::mutate_component(player_id, components::player_yaw(), |yaw| {
            *yaw = (*yaw + msg.mouse_delta.x * 0.01) % std::f32::consts::TAU;
        })
        .unwrap_or_default();

        entity::set_component(player_id, rotation(), Quat::from_rotation_z(yaw));

        let pitch = entity::mutate_component(player_id, components::player_pitch(), |pitch| {
            *pitch = (*pitch + msg.mouse_delta.y * 0.01)
                .clamp(-std::f32::consts::FRAC_PI_3, std::f32::consts::FRAC_PI_3);
        })
        .unwrap_or_default();

        if let Some(cam_id) = entity::get_component(player_id, components::player_cam_ref()) {
            entity::set_component(
                cam_id,
                rotation(),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2 + pitch),
            );
        }

        if msg.shoot {
            messages::Shoot {
                ray_origin: msg.ray_origin,
                ray_dir: msg.ray_dir,
                source: player_id,
            }
            .send_local_broadcast(false);
        }
    });

    query((
        player(),
        components::player_direction(),
        rotation(),
        components::player_vspeed(),
    ))
    .each_frame(move |list| {
        for (player_id, (_, direction, rot, vspeed)) in list {
            let speed = vec2(0.04, 0.06);
            let displace = rot * (direction.normalize_or_zero() * speed).extend(vspeed);
            let collision = physics::move_character(player_id, displace, 0.01, frametime());
            if collision.down {
                entity::set_component(player_id, components::player_vspeed(), 0.0);
            } else {
                entity::mutate_component(player_id, components::player_vspeed(), |vspeed| {
                    *vspeed -= 2.3 * frametime(); // 1/60 second for example
                });
            }
        }
    });
}
