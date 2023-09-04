use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    core::{
        messages::Collision,
        player::components::{is_player, user_id},
        primitives::{
            components::{cube, torus_inner_radius, torus_loops, torus_outer_radius, torus_slices},
            concepts::make_torus,
        },
        rendering::components::color,
        transform::{
            components::{rotation, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use packages::{
    tangent_schema::components::vehicle,
    this::components::{obstacle, score},
};

use crate::packages::this::{components::visited_checkpoint_ids, messages::CheckpointHit};

#[main]
fn main() {
    spawn_query(cube()).bind(|cubes| {
        for (cube, _) in cubes {
            entity::add_component(cube, obstacle(), ());
        }
    });

    spawn_query(is_player()).bind(|vehicles| {
        for (vehicle, _) in vehicles {
            entity::add_component(vehicle, score(), 0);
        }
    });

    Collision::subscribe(|collision| {
        let ids = collision.ids;

        let vehicle = ids
            .iter()
            .copied()
            .find(|id| entity::has_component(*id, vehicle()));
        let obstacle = ids
            .iter()
            .copied()
            .find(|id| entity::has_component(*id, obstacle()));

        if let Some((vehicle, _obstacle)) = vehicle.zip(obstacle) {
            let Some(player_id) = entity::get_component(vehicle, self::vehicle()) else {
                return;
            };
            entity::mutate_component_with_default(player_id, score(), 0, |v| *v -= 10);
        }
    });

    const CHECKPOINT_RADIUS: f32 = 1.5;
    const CHECKPOINT_RADIUS_SQR: f32 = CHECKPOINT_RADIUS * CHECKPOINT_RADIUS;
    const CHECKPOINT_SPACE: f32 = 30.0;
    const CHECKPOINT_DEGREES: f32 = 30.0;
    const CHECKPOINT_SCORE: i32 = 20;

    let mut last_position = vec3(0.0, 0.0, CHECKPOINT_RADIUS);
    let mut last_rotation = Quat::from_rotation_x(-FRAC_PI_2);
    let mut checkpoints = vec![];
    for _ in 0..10 {
        let id = Entity::new()
            .with_merge(make_transformable())
            .with_merge(make_torus())
            .with(torus_inner_radius(), 0.2)
            .with(torus_outer_radius(), CHECKPOINT_RADIUS)
            .with(torus_slices(), 16)
            .with(torus_loops(), 8)
            .with(translation(), last_position)
            .with(rotation(), last_rotation)
            .with(color(), vec4(1.0, 1.0, 0.0, 1.0))
            .spawn();
        checkpoints.push(id);

        last_rotation *= Quat::from_rotation_y(
            (random::<f32>() * CHECKPOINT_DEGREES - CHECKPOINT_DEGREES / 2.).to_radians(),
        );
        last_position += last_rotation * vec3(random(), 0.0, -CHECKPOINT_SPACE);
    }

    query(translation())
        .requires(vehicle())
        .each_frame(move |vehicles| {
            for (id, translation) in vehicles {
                let Some(player_id) = entity::get_component(id, self::vehicle()) else {
                    continue;
                };

                for checkpoint in &checkpoints {
                    let checkpoint_position =
                        entity::get_component(*checkpoint, self::translation()).unwrap_or_default();
                    let distance_sqr = translation.distance_squared(checkpoint_position);
                    if distance_sqr < CHECKPOINT_RADIUS_SQR {
                        let mut added = false;
                        entity::mutate_component_with_default(
                            player_id,
                            visited_checkpoint_ids(),
                            vec![],
                            |ids| {
                                if ids.contains(checkpoint) {
                                    return;
                                }

                                ids.push(*checkpoint);
                                added = true;
                            },
                        );

                        if added {
                            entity::mutate_component_with_default(player_id, score(), 0, |v| {
                                *v += CHECKPOINT_SCORE
                            });

                            CheckpointHit.send_client_targeted_reliable(
                                entity::get_component(player_id, user_id()).unwrap(),
                            );
                        }
                    }
                }
            }
        });
}
