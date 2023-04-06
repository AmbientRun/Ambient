use ambient_api::{
    components::core::{
        physics::{
            character_controller_height, character_controller_radius, physics_controlled,
            plane_collider, sphere_collider, visualizing,
        },
        player::player,
        primitives::{cube, quad},
        rendering::color,
        transform::{rotation, scale, translation},
    },
    concepts::{make_sphere, make_transformable},
    prelude::*,
    rand,
};

use components::{player_mouse_delta_x, player_movement_direction};

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

    spawn_query(player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_transformable())
                    .with_default(cube())
                    .with(color(), rand::random::<Vec3>().extend(1.0))
                    .with(character_controller_height(), 2.)
                    .with(character_controller_radius(), 0.5)
                    .with_default(physics_controlled())
                    .with_default(visualizing()),
            );
        }
    });

    messages::Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        entity::add_component(player_id, player_movement_direction(), msg.direction);
        entity::add_component(player_id, player_mouse_delta_x(), msg.mouse_delta_x);
    });

    query((
        player(),
        player_movement_direction(),
        player_mouse_delta_x(),
        rotation(),
    ))
    .each_frame(move |players| {
        for (player_id, (_, direction, mouse_delta_x, rot)) in players {
            let speed = 0.1;

            let displace = rot * (direction.normalize_or_zero() * speed).extend(-0.1);
            physics::move_character(player_id, displace, 0.01, frametime());

            entity::mutate_component(player_id, rotation(), |x| {
                *x *= Quat::from_rotation_z(mouse_delta_x * 0.01)
            })
            .unwrap_or_default();
        }
    });
}
