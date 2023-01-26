use std::hash::{Hash, Hasher};

use components::core::{
    camera::{aspect_ratio, aspect_ratio_from_window, fovy, near, perspective_infinite_reverse, projection, projection_view},
    ecs::dont_store,
    game_objects::player_camera,
    player::{player, user_id},
    transform::{inv_local_to_world, local_to_world, rotation, translation},
};
use elements_scripting_interface::*;

pub mod components;
pub mod params;

#[main]
pub async fn main() -> EventResult {
    const RADIUS: f32 = 5.0;

    // When a player spawns, give them a camera.
    spawn_query((player(), user_id())).bind(|players| {
        for (_, (_, player_user_id)) in players {
            entity::game_object_base()
                .with(user_id(), player_user_id)
                .with_default(player_camera())
                .with_default(dont_store())
                .with(translation(), vec3(0.0, RADIUS, 0.0))
                .with(rotation(), Quat::from_rotation_x(90.0f32.to_radians()))
                .with_default(local_to_world())
                .with_default(inv_local_to_world())
                .with(near(), 0.1)
                .with(fovy(), 1.0)
                .with(perspective_infinite_reverse(), ())
                .with(aspect_ratio(), 1.)
                .with(aspect_ratio_from_window(), ())
                .with_default(projection())
                .with_default(projection_view())
                .spawn(false);
        }
    });

    // When a player despawns, delete their camera.
    let camera_query = query((player_camera(), user_id())).build();
    despawn_query((player(), user_id())).bind(move |players| {
        let player_cameras = camera_query.evaluate();
        for (_, (_, user_id)) in players {
            if let Some((id, _)) = player_cameras.iter().find(|(_, (_, camera_user_id))| user_id == *camera_user_id) {
                entity::despawn(*id);
            }
        }
    });

    camera_query.bind(|cameras| {
        for (id, (_, player_user_id)) in cameras {
            let offset = {
                // Very simple way to uniquely offset each player's time
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                player_user_id.hash(&mut hasher);
                (hasher.finish() % 2u64.pow(16)) as f32
            };
            let seconds_per_revolution = 5.0;
            let rotation_in_revolutions = time() * (1.0 / seconds_per_revolution) + offset;
            let radians = rotation_in_revolutions * std::f32::consts::TAU;

            entity::set_position(id, Quat::from_rotation_z(radians) * (Vec3::Y * RADIUS));
            entity::set_rotation(id, Quat::from_rotation_z(radians) * Quat::from_rotation_x(90.0f32.to_radians()));
        }
    });

    loop {
        println!("Hello, world! It is {}", time());
        sleep(0.5).await;
    }
}
