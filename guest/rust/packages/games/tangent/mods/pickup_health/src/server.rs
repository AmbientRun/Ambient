use ambient_api::{
    core::{
        model::components::model_from_url,
        transform::components::{rotation, translation},
    },
    prelude::*,
};
use packages::{
    game_object::components::{health, max_health},
    this::{concepts::HealthPickup, messages::OnHealthPickup},
};

#[main]
pub fn main() {
    spawn_query(HealthPickup::as_query()).bind(|pickups| {
        for (id, _) in pickups {
            entity::add_component(
                id,
                model_from_url(),
                packages::this::assets::url("Low Poly Medkit 3.fbx/models/main.json"),
            );
        }
    });

    let candidate_query = query((translation(), health(), max_health())).build();

    query(HealthPickup::as_query()).each_frame(move |pickups| {
        let candidates = candidate_query.evaluate();

        for (id, pickup) in pickups {
            entity::set_component(
                id,
                rotation(),
                Quat::from_rotation_z(game_time().as_secs_f32()),
            );

            for (entity_id, (translation, health, max_health)) in candidates.iter().copied() {
                if pickup.translation.distance_squared(translation) > 3f32.powi(2) {
                    continue;
                }

                if health == 0.0 {
                    continue;
                }

                let new_health = (health + 25.0).clamp(0.0, max_health);
                if health == new_health {
                    continue;
                }

                entity::set_component(entity_id, self::health(), new_health);
                entity::despawn_recursive(id);

                OnHealthPickup {
                    position: pickup.translation,
                }
                .send_client_broadcast_unreliable();
            }
        }
    });
}
