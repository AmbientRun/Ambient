use ambient_api::{
    core::{ecs::components::remove_at_game_time, transform::components::translation},
    prelude::*,
};
use packages::{game_object::components::health, this::concepts::Explosion};

#[main]
pub fn main() {
    let vehicle_query = query(translation()).requires(health()).build();

    spawn_query(Explosion::as_query()).bind(move |explosions| {
        for (id, explosion) in explosions {
            let Explosion {
                radius,
                translation,
                damage,
                ..
            } = explosion;

            entity::add_component(
                id,
                remove_at_game_time(),
                game_time() + Duration::from_secs(2),
            );

            physics::add_radial_impulse(
                translation,
                damage * 50.0,
                radius,
                physics::FalloffRadius::FalloffToZeroAt(radius),
            );

            for (go_id, go_translation) in vehicle_query.evaluate() {
                let distance = go_translation.distance(translation);
                if distance > radius {
                    continue;
                }

                let closeness = (radius - distance) / radius;
                entity::mutate_component(go_id, health(), |health| {
                    *health = (*health - closeness * damage).max(0.0);
                });
            }
        }
    });
}
