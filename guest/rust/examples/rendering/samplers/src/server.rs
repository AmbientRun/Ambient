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
pub fn main() {
    Entity::new()
        .with_merge(make_PerspectiveInfiniteReverseCamera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(main_scene(), ())
        .with(translation(), vec3(2.0, 2.0, 4.0))
        .with(lookat_target(), vec3(0.0, 0.0, 0.0))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(translation(), vec3(-1.25, 0.0, 0.0))
        .with(prefab_from_url(), assets::url("quad-linear.glb"))
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(translation(), vec3(1.25, 0.0, 0.0))
        .with(prefab_from_url(), assets::url("quad-nearest.glb"))
        .spawn();
}
