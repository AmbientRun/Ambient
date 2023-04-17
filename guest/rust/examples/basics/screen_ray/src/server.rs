use ambient_api::{
    components::core::{
        physics::plane_collider,
        primitives::{cube, quad},
        transform::translation,
    },
    concepts::make_transformable,
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with_default(plane_collider())
        .spawn();

    let cube_id = Entity::new()
        .with_merge(make_transformable())
        .with_default(cube())
        .spawn();

    messages::Input::subscribe(move |_source, msg| {
        if let Some(hit) = physics::raycast_first(msg.ray_origin, msg.ray_dir) {
            // Set position of cube to the raycast hit position
            entity::set_component(cube_id, translation(), hit.position);
        }
    });
}
