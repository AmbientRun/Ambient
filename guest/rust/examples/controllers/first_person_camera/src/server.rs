use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        ecs::components::{children, parent},
        physics::components::{
            character_controller_height, character_controller_radius, physics_controlled,
            plane_collider, sphere_collider,
        },
        player::components::{is_player, user_id},
        primitives::{
            components::{cube, quad},
            concepts::make_sphere,
        },
        rendering::components::color,
        transform::{
            components::{local_to_parent, rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};
use embers::ambient_example_first_person_camera::{
    components::{ball_ref, player_head_ref, player_movement_direction, player_pitch, player_yaw},
    messages::Input,
};

use std::f32::consts::{PI, TAU};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_transformable())
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .with(plane_collider(), ())
        .spawn();

    let ball = Entity::new()
        .with_merge(make_transformable())
        .with_merge(make_sphere())
        .with(sphere_collider(), 0.5)
        .with(translation(), vec3(5., 5., 1.))
        .spawn();

    spawn_query((is_player(), user_id())).bind(move |players| {
        for (id, (_, uid)) in players {
            let head = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with(aspect_ratio_from_window(), EntityId::resources())
                .with(main_scene(), ())
                .with(user_id(), uid)
                .with(translation(), Vec3::Z * 2.)
                .with(parent(), id)
                .with(local_to_parent(), Default::default())
                .with(rotation(), Quat::from_rotation_x(PI / 2.))
                .spawn();
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_transformable())
                    .with(cube(), ())
                    .with(physics_controlled(), ())
                    .with(player_movement_direction(), Vec2::default())
                    .with(color(), Vec4::ONE)
                    .with(character_controller_height(), 2.)
                    .with(character_controller_radius(), 0.5)
                    .with(player_head_ref(), head)
                    .with(ball_ref(), ball)
                    .with(children(), vec![head])
                    .with(player_pitch(), 0.0)
                    .with(player_yaw(), 0.0),
            );
        }
    });

    Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        entity::set_component(player_id, player_movement_direction(), msg.direction);

        let yaw = entity::mutate_component(player_id, player_yaw(), |yaw| {
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

    query((is_player(), player_movement_direction(), rotation())).each_frame(move |players| {
        for (player_id, (_, direction, rot)) in players {
            let speed = 0.1;

            let displace = rot * (direction.normalize_or_zero() * speed).extend(-0.1);
            physics::move_character(player_id, displace, 0.01, delta_time());
        }
    });
}
