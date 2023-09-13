use ambient_api::prelude::*;

#[main]
pub fn main() {
    make_my_camera_foggy();
    let sun = make_my_local_sun_with_sky();

    // spawn_query(is_player()).bind(move |players| {
    //     for (id, _) in players {
    //         // Only attach models to other players
    //         if id != player::get_local() {
    //             add_components(
    //                 id,
    //                 Entity::new()
    //                     .with(model_from_url(), base_assets::assets::url("Y Bot.fbx"))
    //                     .with(basic_character_animations(), id),
    //             );
    //         }
    //     }
    // });
    pulse_suns_fog_over_time(sun);
}

pub fn pulse_suns_fog_over_time(sun: EntityId) {
    use ambient_api::core::rendering::components::{fog_color, fog_density};
    use packages::this::components::coldness;
    ambient_api::core::messages::Frame::subscribe(move |_| {
        let coldness: f32 = entity::get_component(player::get_local(), coldness()).unwrap_or(1.);
        // let coldness: f32 = 0.5 + 0.5 * game_time().as_secs_f32().sin();
        if coldness < 0.60 {
            let t = coldness / 0.60;
            entity::mutate_component(sun, fog_density(), |foggy| {
                *foggy = *foggy * 0.9 + 0.1 * (0.02 + 0.18 * t);
            });
        } else {
            let t = (coldness - 0.60) / (1. - 0.60);
            entity::set_component(sun, fog_density(), 0.20 + 0.80 * t * t);
        }
        entity::mutate_component(sun, fog_color(), |color| {
            *color = color.lerp(
                vec3(0.75, 0.45, 0.75).lerp(vec3(0.60, 1.00, 1.00), coldness.sqrt()),
                0.1,
            )
        });
    });
}

pub fn make_my_camera_foggy() {
    use ambient_api::core::camera::{
        components::fog, concepts::perspective_infinite_reverse_camera,
    };
    spawn_query(())
        .requires(perspective_infinite_reverse_camera())
        .bind(|cameras| {
            for (camera, _) in cameras {
                entity::add_component(camera, fog(), ());
            }
        });
}

pub fn make_my_local_sun_with_sky() -> EntityId {
    use ambient_api::core::{
        app::components::main_scene,
        rendering::components::{
            fog_color, fog_density, fog_height_falloff, light_diffuse, sky, sun,
        },
        transform::{components::rotation, concepts::make_transformable},
    };

    Entity::new()
        .with_merge(make_transformable())
        .with(sky(), ())
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with(sun(), 0.0)
        .with(rotation(), Default::default())
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        // .with(fog_color(), vec3(0.88, 0.37, 0.34)) // dusty red
        .with(fog_color(), vec3(0.34, 0.37, 0.88)) // blueish. cold.
        // .with(fog_color(), vec3(0., 0., 0.))
        .with(fog_density(), 0.1)
        .with(fog_height_falloff(), 0.01)
        .with(rotation(), Quat::from_rotation_y(190.0f32.to_radians()))
        .spawn()
}
