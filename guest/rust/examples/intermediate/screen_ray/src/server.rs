use ambient_api::{
    core::{
        physics::components::{cube_collider, plane_collider},
        player::components::is_player,
        primitives::components::{cube, quad},
        rendering::components::color,
        transform::components::{local_to_world, translation},
    },
    prelude::*,
};
use packages::this::{components::player_cube_ref, messages::Input};

pub mod packages;

#[main]
pub fn main() {
    Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(plane_collider(), ())
        .with(quad(), ())
        .spawn();

    spawn_query(is_player()).bind(|players| {
        for (player_id, _) in players {
            let player_color = random::<Vec3>().extend(1.0);
            entity::add_component(
                player_id,
                player_cube_ref(),
                spawn_cube(Vec3::ZERO, player_color, false),
            );
        }
    });

    despawn_query(player_cube_ref())
        .requires(is_player())
        .bind(|players| {
            for (_, cube_id) in players {
                entity::despawn(cube_id);
            }
        });

    Input::subscribe(move |ctx, msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };

        let Some(cube_id) = entity::get_component(player_id, player_cube_ref()) else {
            return;
        };

        let Some(color) = entity::get_component(cube_id, color()) else {
            return;
        };

        let Some(hit) = physics::raycast_first(msg.ray_origin, msg.ray_dir) else {
            return;
        };

        // Offset the cube's half-height so that it's resting on the ground
        let position = hit.position + Vec3::Z * 0.5;

        // Set position of cube to the raycast hit position
        entity::set_component(cube_id, translation(), position);

        if msg.spawn {
            spawn_cube(position, color, true);
        }
    });
}

fn spawn_cube(position: Vec3, player_color: Vec4, with_collider: bool) -> EntityId {
    let mut entity = Entity::new()
        .with(translation(), position)
        .with(cube(), ())
        .with(color(), player_color);
    if with_collider {
        entity.set(cube_collider(), Vec3::ONE);
    }
    entity.spawn()
}
