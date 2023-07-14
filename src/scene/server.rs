use ambient_api::{
    components::core::{
        physics::{cube_collider, plane_collider, sphere_collider, visualize_collider},
        prefab::prefab_from_url,
        primitives::{cube, quad},
        transform::scale,
    },
    concepts::{make_sphere, make_transformable},
    entity::{add_components, remove_components, wait_for_component},
    prelude::*,
};

#[main]
pub async fn main() {
    Entity::new()
        .with_default(quad())
        .with_default(plane_collider())
        .with(scale(), Vec3::ONE * 50.)
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with(
            prefab_from_url(),
            asset::url("assets/map/fps_map_ghost_city.glb").unwrap(),
        )
        .spawn();
}
