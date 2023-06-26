use ambient_api::{
    components::core::{
        physics::{cube_collider, plane_collider, sphere_collider, visualize_collider},
        primitives::{cube, quad},
    },
    concepts::{make_sphere, make_transformable},
    entity::{add_components, remove_components},
    prelude::*,
};

#[main]
pub fn main() {
    // messages::StartGame::subscribe(|_, msg| {
    Entity::new()
        .with_default(quad())
        .with_default(plane_collider())
        .with(scale(), Vec3::ONE * 30.)
        .spawn();
    // });
}
