use ambient_api::{
    core::{
        app::components::main_scene,
        physics::components::{
            character_controller_height, character_controller_radius, physics_controlled,
            plane_collider, sphere_collider,
        },
        player::components::is_player,
        primitives::{
            components::{cube, quad},
            concepts::make_sphere,
        },
        rendering::components::{color, fog_density, light_diffuse, sky, sun},
        transform::{
            components::{rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};
use embers::ambient_example_third_person_camera::{
    components::{
        camera_follow_distance, player_mouse_delta_x, player_movement_direction, player_scroll,
    },
    messages::Input,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_transformable())
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.0)
        .with(color(), vec4(1.0, 0.0, 0.0, 1.0))
        .with(plane_collider(), ())
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_merge(make_sphere())
        .with(color(), vec4(0.5, 0.0, 1.0, 1.0))
        .with(sphere_collider(), 0.5)
        .with(translation(), vec3(5.0, 5.0, 1.0))
        .spawn();

    // Spawn a sun
    make_transformable()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-1.0))
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.001)
        .with(main_scene(), ())
        .spawn();

    // And an atmosphere to go with id
    make_transformable().with(sky(), ()).spawn();

    spawn_query(is_player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_transformable())
                    .with(player_movement_direction(), Vec2::ZERO)
                    .with(player_mouse_delta_x(), 0.0)
                    .with(player_scroll(), 0.0)
                    .with(physics_controlled(), ())
                    .with(cube(), ())
                    .with(color(), Vec4::ONE)
                    .with(character_controller_height(), 2.0)
                    .with(character_controller_radius(), 0.5)
                    .with(camera_follow_distance(), 4.0),
            );
        }
    });

    Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        entity::set_component(player_id, player_movement_direction(), msg.direction);
        entity::set_component(player_id, player_mouse_delta_x(), msg.mouse_delta_x);
        entity::set_component(player_id, player_scroll(), msg.scroll);
    });

    query((
        is_player(),
        player_movement_direction(),
        player_mouse_delta_x(),
        player_scroll(),
        rotation(),
    ))
    .each_frame(move |players| {
        for (player_id, (_, direction, mouse_delta_x, scroll, rot)) in players {
            let speed = 0.1;

            let displace = rot * (direction.normalize_or_zero() * speed).extend(-0.1);
            physics::move_character(player_id, displace, 0.01, delta_time());

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
