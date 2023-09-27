use ambient_api::{
    core::{
        ecs::components::remove_at_game_time,
        messages::ModuleUnload,
        physics::components::{density, dynamic, linear_velocity, sphere_collider},
        player::components::is_player,
        primitives::concepts::Sphere,
        rendering::components::{
            color, fog_color, fog_density, fog_height_falloff, light_diffuse, sun,
        },
        transform::components::translation,
    },
    prelude::*,
    rand,
};
use packages::tangent_schema::player::components::{character_ref, vehicle_ref};

#[main]
pub fn main() {
    let sun_id = query(sun())
        .build()
        .evaluate()
        .into_iter()
        .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Less))
        .map(|v| v.0);

    let Some(sun_id) = sun_id else {
        return;
    };

    let old_sun = entity::get_all_components(sun_id);

    let new_color = vec3(247.0 / 255.0, 55.0 / 255.0, 24.0 / 255.0);
    entity::add_component(sun_id, light_diffuse(), new_color);
    entity::add_component(sun_id, fog_color(), new_color);
    entity::add_component(sun_id, fog_density(), 0.05);
    entity::add_component(sun_id, fog_height_falloff(), 0.01);

    let players_query = query(is_player()).build();

    fixed_rate_tick(Duration::from_millis(50), move |_| {
        for (player_id, _) in players_query.evaluate() {
            let Some(player_position) = entity::get_component(player_id, vehicle_ref())
                .or_else(|| entity::get_component(player_id, character_ref()))
                .and_then(|e| entity::get_component(e, translation()))
            else {
                continue;
            };

            let distribution = 40.0;
            let mut rng = rand::thread_rng();
            let base_offset = vec3(0.0, -100.0, 100.0);
            let offset = base_offset
                + vec3(
                    rng.gen_range(-distribution..=distribution),
                    rng.gen_range(-distribution..=distribution),
                    rng.gen_range(-distribution..=distribution),
                );

            let position = player_position + offset;

            let radius = rng.gen_range(0.2..=1.0);

            Sphere {
                sphere_radius: radius,
                ..Sphere::suggested()
            }
            .make()
            .with(color(), new_color.extend(1.0))
            .with(sphere_collider(), radius)
            .with(density(), 40.0)
            .with(translation(), position)
            .with(dynamic(), true)
            .with(linear_velocity(), -offset * 0.2)
            .with(remove_at_game_time(), game_time() + Duration::from_secs(10))
            .spawn();
        }
    });

    ModuleUnload::subscribe(move |_| {
        entity::remove_component(sun_id, light_diffuse());
        entity::remove_component(sun_id, fog_color());
        entity::remove_component(sun_id, fog_density());
        entity::remove_component(sun_id, fog_height_falloff());
        entity::add_components(sun_id, old_sun.clone());
    });
}
