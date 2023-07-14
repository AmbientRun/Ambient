use ambient_api::{
    components::core::{
        app::main_scene,
        physics::{cube_collider, plane_collider, sphere_collider, visualize_collider},
        prefab::prefab_from_url,
        primitives::{cube, quad},
        rendering::{fog_density, light_diffuse, sky, sun},
        transform::{rotation, scale},
    },
    concepts::{make_sphere, make_transformable},
    entity::{add_components, remove_components, wait_for_component},
    prelude::*,
};

#[main]
pub async fn main() {
    Entity::new()
        // .with_default(quad())
        .with_default(plane_collider())
        .with(scale(), Vec3::ONE * 50.)
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with(
            prefab_from_url(),
            // asset::url("assets/map/map_collider.glb").unwrap(),
            asset::url("assets/map/fps_map_ghost_city.glb").unwrap(),
        )
        .with(scale(), Vec3::ONE * 1.5)
        .spawn();
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with(rotation(), Quat::from_rotation_y(-0.6))
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.1)
        .spawn();
}
