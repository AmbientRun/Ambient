use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window, concepts::make_PerspectiveInfiniteReverseCamera,
        },
        prefab::components::prefab_from_url,
        primitives::components::quad,
        rendering::components::cast_shadows,
        transform::{
            components::{lookat_target, scale, translation},
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
        .with(translation(), Vec3::ONE * 5. + Vec3::Z * 1.5)
        .with(lookat_target(), Vec3::Z * 1.5)
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 100.)
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(cast_shadows(), ())
        .with(
            prefab_from_url(),
            assets::url("TreePack/Prefabs/Standard/Fir_01_Plant.prefab"),
        )
        .spawn();
}
