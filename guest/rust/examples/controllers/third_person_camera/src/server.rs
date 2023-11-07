use ambient_api::{
    core::{
        app::components::main_scene,
        model::components::model_from_url,
        physics::components::{plane_collider, sphere_collider},
        player::components::is_player,
        primitives::{components::quad, concepts::Sphere},
        rendering::components::{color, fog_density, light_diffuse, sky, sun},
        transform::components::{rotation, scale, translation},
    },
    prelude::*,
};
use packages::{
    base_assets,
    character_animation::components::{basic_character_animations, use_rifle_animations},
    nameplates::components::height_offset,
    third_person_controller::concepts::ThirdPersonController,
};

const USE_RIFLE_ANIMATIONS: bool = false;

#[main]
pub fn main() {
    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.0)
        .with(color(), vec4(1.0, 0.0, 0.0, 1.0))
        .with(plane_collider(), ())
        .spawn();

    Entity::new()
        .with_merge(Sphere::suggested())
        .with(color(), vec4(0.5, 0.0, 1.0, 1.0))
        .with(sphere_collider(), 0.5)
        .with(translation(), vec3(5.0, 5.0, 1.0))
        .spawn();

    // Spawn a sun
    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Quat::from_rotation_y(-1.0))
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.001)
        .with(main_scene(), ())
        .spawn();

    // And an atmosphere to go with it
    Entity::new().with(sky(), ()).spawn();

    spawn_query(is_player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(ThirdPersonController::suggested())
                    .with(model_from_url(), base_assets::assets::url("Y Bot.fbx"))
                    .with(basic_character_animations(), id)
                    .with(height_offset(), 2.0),
            );

            if USE_RIFLE_ANIMATIONS {
                entity::add_component(id, use_rifle_animations(), ());
            }
        }
    });
}
