use ambient_api::{
    core::{
        physics::components::plane_collider,
        primitives::components::{cube, quad},
        transform::{components::translation, concepts::make_transformable},
    },
    prelude::*,
};
use packages::this::messages::{Input, WorldPosition};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_transformable())
        .with(quad(), ())
        .with(plane_collider(), ())
        .spawn();

    let cube_id = Entity::new()
        .with_merge(make_transformable())
        .with(cube(), ())
        .spawn();

    Input::subscribe(move |_source, msg| {
        if let Some(hit) = physics::raycast_first(msg.ray_origin, msg.ray_dir) {
            // Set position of cube to the raycast hit position
            entity::set_component(cube_id, translation(), hit.position);
            WorldPosition::new(hit.position).send_client_broadcast_unreliable();
        }
    });
}
