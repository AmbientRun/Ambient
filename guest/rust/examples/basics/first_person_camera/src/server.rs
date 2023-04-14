use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        ecs::{children, parent},
        physics::{
            character_controller_height, character_controller_radius, physics_controlled,
            plane_collider, sphere_collider, visualizing,
        },
        player::{player, user_id},
        primitives::{cube, quad},
        rendering::color,
        transform::{local_to_parent, rotation, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};

use components::{player_head_ref, player_mouse_delta, player_movement_direction};
use std::f32::consts::PI;

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .with_default(plane_collider())
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_merge(make_sphere())
        .with(sphere_collider(), 1.)
        .with(translation(), vec3(5., 5., 1.))
        .with_default(visualizing())
        .spawn();

    spawn_query((player(), user_id())).bind(move |players| {
        for (id, (_, uid)) in players {
            let head = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with(aspect_ratio_from_window(), EntityId::resources())
                .with_default(main_scene())
                .with(user_id(), uid)
                .with(translation(), Vec3::Z * 2.)
                .with(parent(), id)
                .with_default(local_to_parent())
                .with(rotation(), Quat::from_rotation_x(PI / 2.))
                .spawn();
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_transformable())
                    .with_default(cube())
                    .with(color(), Vec4::ONE)
                    .with(character_controller_height(), 2.)
                    .with(character_controller_radius(), 0.5)
                    .with_default(physics_controlled())
                    .with_default(visualizing())
                    .with(player_head_ref(), head)
                    .with(children(), vec![head]),
            );
        }
    });

    messages::Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        entity::add_component(player_id, player_movement_direction(), msg.direction);
        entity::add_component(player_id, player_mouse_delta(), msg.mouse_delta);
    });

    query((
        player(),
        player_head_ref(),
        player_movement_direction(),
        player_mouse_delta(),
        rotation(),
    ))
    .each_frame(move |players| {
        for (player_id, (_, head, direction, mouse_delta, rot)) in players {
            let speed = 0.1;

            let displace = rot * (direction.normalize_or_zero() * speed).extend(-0.1);
            physics::move_character(player_id, displace, 0.01, frametime());

            entity::mutate_component(player_id, rotation(), |x| {
                *x *= Quat::from_rotation_z(mouse_delta.x * 0.01);
            })
            .unwrap_or_default();
            entity::mutate_component(head, rotation(), |x| {
                *x *= Quat::from_rotation_x(mouse_delta.y * 0.01);
            });
        }
    });
}
