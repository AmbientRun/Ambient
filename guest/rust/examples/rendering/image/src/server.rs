use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window, concepts::make_PerspectiveInfiniteReverseCamera,
        },
        primitives::components::quad,
        rendering::components::pbr_material_from_url,
        transform::{
            components::{lookat_target, scale, translation},
            concepts::make_Transformable,
        },
    },
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_PerspectiveInfiniteReverseCamera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(translation(), vec3(5., 5., 6.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .with(main_scene(), ())
        .spawn();

    Entity::new()
        .with_merge(make_Transformable())
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.)
        .with(
            pbr_material_from_url(),
            packages::this::assets::url("pipeline.toml/0/mat.json"),
        )
        .spawn();
}
