use std::f32::consts::PI;

use ambient_api::{
    components::core::{
        app::main_scene,
        physics::{angular_velocity, linear_velocity},
        player::player as player_component,
        prefab::prefab_from_url,
        rendering::{cast_shadows, fog_density, light_diffuse, sky, sun, water},
        transform::{rotation, scale, translation},
    },
    concepts::make_transformable,
    messages::Frame,
    prelude::*,
};
use components::player_vehicle;

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(water())
        .with(scale(), Vec3::ONE * 2000.)
        .spawn();

    make_sun();

    spawn_query(player_component()).bind(|players| {
        for (player, ()) in players {
            let vehicle = Entity::new()
                .with_merge(make_transformable())
                .with(
                    prefab_from_url(),
                    asset::url("assets/models/raceCarWhite.glb").unwrap(),
                )
                .with_default(cast_shadows())
                .with_default(linear_velocity())
                .with_default(angular_velocity())
                .with(translation(), vec3(0., 0., 5.))
                .spawn();

            entity::add_component(player, player_vehicle(), vehicle);
        }
    });

    despawn_query(player_component()).bind(|players| {
        for (player, ()) in players {
            if let Some(vehicle) = entity::get_component(player, player_vehicle()) {
                entity::despawn(vehicle);
            }
        }
    });
}

fn make_sun() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();

    let sun = Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with_default(rotation())
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.0)
        .spawn();

    Frame::subscribe(move |_| {
        // How long a full cycle takes.
        const HALF_DAY_LENGTH: f32 = 30.0;

        entity::set_component(
            sun,
            rotation(),
            Quat::from_rotation_y(PI + PI * (time() * PI / HALF_DAY_LENGTH).sin().abs()),
        );
    });
}
