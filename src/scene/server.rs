use ambient_api::{
    components::core::{
        app::main_scene,
        physics::plane_collider,
        prefab::prefab_from_url,
        primitives::quad,
        rendering::{fog_density, light_diffuse, sky, sun},
        transform::{rotation, scale},
    },
    concepts::make_transformable,
    prelude::*,
};

#[main]
pub async fn main() {
    Entity::new()
        .with_default(quad())
        .with_default(plane_collider())
        .with(scale(), Vec3::ONE * 1.)
        .spawn();
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();
    Entity::new()
        .with_merge(make_transformable())
        .with(
            prefab_from_url(),
            // asset::url("assets/map/c0.glb").unwrap(),
            asset::url("assets/map/fps_map_ghost_city.glb").unwrap(),
        )
        .with(scale(), Vec3::ONE * 1.5)
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
