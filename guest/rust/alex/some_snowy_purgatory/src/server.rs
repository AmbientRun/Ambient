use ambient_api::{
    core::{
        app::components::name,
        // camera::concepts::{
        //     PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        // },
        // player::components::is_player,
        transform::components::translation,
    },
    prelude::*,
};

use packages::{
    temperature::components::{
        temperature_src_falloff, temperature_src_radius, temperature_src_rate,
    },
    this::components::ambient_loop,
};

#[main]
pub fn main() {
    // SHOULD BE GIVEN BY TEMPERATURE SCHEMA
    // const DEATH_TEMP: f32 = 21.13;
    // const NORMAL_TEMP: f32 = 36.65;

    // // temp camera - quickly overwritten by clientside camera
    // PerspectiveInfiniteReverseCamera {
    //     optional: PerspectiveInfiniteReverseCameraOptional {
    //         aspect_ratio_from_window: Some(entity::resources()),
    //         main_scene: Some(()),
    //         translation: Some(Vec3::ONE * 5.),
    //         ..default()
    //     },
    //     ..PerspectiveInfiniteReverseCamera::suggested()
    // }
    // .make()
    // .with(lookat_target(), vec3(0., 0., 0.))
    // .spawn();

    // cold storm
    Entity::new()
        .with(translation(), Vec3::ZERO)
        .with(temperature_src_rate(), -0.33)
        .with(temperature_src_radius(), core::f32::MAX)
        .spawn();

    // campfire(s) ('fireplace')
    spawn_query((name(), translation())).bind(|ents| {
        for (ent, (entname, pos)) in ents {
            if entname == "mt-fireplace" {
                entity::add_components(
                    ent,
                    Entity::new()
                        .with(temperature_src_rate(), 5.0)
                        .with(temperature_src_radius(), 20.0)
                        .with(
                            ambient_loop(),
                            packages::this::assets::url("4211__dobroide__firecrackling.ogg"),
                        )
                        // add: droplets: fire
                        .with(
                            packages::snowy_droplets::components::is_droplet_spawner(),
                            (),
                        )
                        .with(packages::snowy_droplets::components::spawns_fire(), ()),
                );
                // add: too hot in center
                Entity::new()
                    .with(translation(), pos) // same location
                    .with(temperature_src_rate(), 20.0) // enough to overwhelm your natural max cooling rate
                    .with(temperature_src_radius(), 6.0) // significantly smaller radius
                    .with(temperature_src_falloff(), 0.75) // slightly longer falloff.
                    .spawn();
            }
        }
    });
}
