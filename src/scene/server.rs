/// Scene is running on the server.
/// it determines the scene of the game.
/// loading assets like maps
/// and places necessary colliders
///
#[allow(unused_imports)]
use ambient_api::{
    components::core::{
        physics::{cube_collider, plane_collider, sphere_collider, visualize_collider},
        primitives::{quad, sphere},
    },
    concepts::make_sphere,
    entity::{add_components, remove_components},
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_default(quad())
        .with_default(plane_collider())
        .with(scale(), Vec3::ONE * 10.)
        .spawn();
}
