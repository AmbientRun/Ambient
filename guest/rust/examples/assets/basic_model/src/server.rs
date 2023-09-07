use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window, concepts::make_PerspectiveInfiniteReverseCamera,
        },
        prefab::components::prefab_from_url,
        transform::{
            components::{lookat_target, translation},
            concepts::make_Transformable,
        },
    },
    prelude::*,
};

use packages::this::assets;

#[main]
pub async fn main() {
    // Camera
    Entity::new()
        .with_merge(make_PerspectiveInfiniteReverseCamera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(main_scene(), ())
        .with(translation(), vec3(2., 2., 2.))
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    // Model
    Entity::new()
        .with_merge(make_Transformable())
        .with(prefab_from_url(), assets::url("Cube.glb"))
        .spawn();
}
