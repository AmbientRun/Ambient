use ambient_api::{
    core::{
        physics::components::plane_collider,
        primitives::components::{cube, quad},
        transform::components::{local_to_world, translation},
    },
    prelude::*,
};
use packages::this::messages::{Input, WorldPosition};

#[main]
pub fn main() {
    Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(plane_collider(), ())
        .with(quad(), ())
        .spawn();

    let cube_id = Entity::new()
        .with(translation(), Vec3::ZERO)
        .with(cube(), ())
        .spawn();

    Input::subscribe(move |_ctx, msg| {
        if let Some(hit) = physics::raycast_first(msg.ray_origin, msg.ray_dir) {
            // Set position of cube to the raycast hit position
            entity::set_component(cube_id, translation(), hit.position);
            WorldPosition::new(hit.position).send_client_broadcast_unreliable();
        }
    });
}
