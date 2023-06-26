use std::f64::consts::FRAC_PI_3;

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
    spawn_query(player()).bind(|results| {
        for (id, _) in results {
            entity::add_component(id, components::player_yaw(), 0.0);
            entity::add_component(id, components::player_pitch(), 0.0);
        }
    });
    messages::Input::subscribe(|source, msg| {
        // receive movement and send this for further processing
        let player_id = source.client_entity_id().unwrap();
        let direction = msg.direction;

        entity::add_component(player_id, components::player_direction(), direction);
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

        println!("Received input: {:?}", msg);
    });

    query((player(), components::player_direction(), rotation())).each_frame(move |list| {
        for (player_id, (_, direction, rot)) in list {
            let speed = 0.1;
            let displace = rot * (direction.normalize_or_zero() * speed).extend(-0.1);
            physics::move_character(player_id, displace, 0.01, frametime());
        }
    });
}
