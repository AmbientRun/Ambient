use ambient_api::{
    core::{
        app::components::main_scene,
        model::components::model_from_url,
        physics::components::{plane_collider, sphere_collider},
        player::components::is_player,
        primitives::{components::quad, concepts::make_Sphere},
        rendering::components::{color, fog_density, light_diffuse, sky, sun},
        transform::{
            components::{rotation, scale, translation},
            concepts::make_Transformable,
        },
    },
    prelude::*,
};
use packages::{
    base_assets, character_animation::components::basic_character_animations,
    fps_controller::components::use_fps_controller,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_Transformable())
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.0)
        .with(color(), vec4(1.0, 0.0, 0.0, 1.0))
        .with(plane_collider(), ())
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with_merge(make_Sphere())
        .with(color(), vec4(0.5, 0.0, 1.0, 1.0))
        .with(sphere_collider(), 0.5)
        .with(translation(), vec3(5.0, 5.0, 1.0))
        .spawn();

    // Spawn a sun
    make_Transformable()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-1.0))
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.001)
        .with(main_scene(), ())
        .spawn();

    // And an atmosphere to go with id
    make_Transformable().with(sky(), ()).spawn();

    spawn_query(is_player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with(use_fps_controller(), ())
                    .with(model_from_url(), base_assets::assets::url("Y Bot.fbx"))
                    .with(basic_character_animations(), id),
            );
        }
    });
}
