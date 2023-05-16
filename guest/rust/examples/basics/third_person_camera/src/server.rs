use ambient_api::{
    components::core::{
        physics::{
            character_controller_height, character_controller_radius, physics_controlled,
            plane_collider, sphere_collider,
        },
        player::player,
        primitives::{cube, quad},
        rendering::color,
        transform::{rotation, scale, translation},
    },
    concepts::{make_sphere, make_transformable},
    prelude::*,
};

use components::*;

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
        .with(sphere_collider(), 0.5)
        .with(translation(), vec3(5., 5., 1.))
        .spawn();

    // Spawn a sun
    make_transformable()
        .with_default(sun())
        .with(rotation(), Quat::from_rotation_y(0.0))
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.001)
        .with_default(main_scene())
        .spawn();

    // And an atmosphere to go with id
    make_transformable().with_default(sky()).spawn();

    query((sun(), rotation())).each_frame(|sun| {
        let elapsed = time();
        for (id, _) in sun {
            entity::mutate_component(id, rotation(), |x| {
                *x = Quat::from_axis_angle(vec3(0.0, 1.0, 0.5).normalize(), elapsed * 0.1)
            })
            .unwrap();
        }
    });

    spawn_query(player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_transformable())
                    .with_default(cube())
                    .with(color(), Vec4::ONE)
                    .with(character_controller_height(), 2.)
                    .with(character_controller_radius(), 0.5)
                    .with(camera_follow_distance(), 4.0)
                    .with_default(physics_controlled()),
            );
        }
    });

    messages::Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        entity::add_component(player_id, player_movement_direction(), msg.direction);
        entity::add_component(player_id, player_mouse_delta_x(), msg.mouse_delta_x);
        entity::add_component(player_id, player_scroll(), msg.scroll);
    });

    query((
        player(),
        player_movement_direction(),
        player_mouse_delta_x(),
        player_scroll(),
        rotation(),
    ))
    .each_frame(move |players| {
        for (player_id, (_, direction, mouse_delta_x, scroll, rot)) in players {
            let speed = 0.1;

            let displace = rot * (direction.normalize_or_zero() * speed).extend(-0.1);
            physics::move_character(player_id, displace, 0.01, frametime());

            entity::mutate_component(player_id, rotation(), |x| {
                *x *= Quat::from_rotation_z(mouse_delta_x * 0.01)
            })
            .unwrap_or_default();

            entity::add_component(player_id, camera_follow_distance(), {
                ((scroll * 0.005) + 5.0).max(1.0)
            });
        }
    });
}
