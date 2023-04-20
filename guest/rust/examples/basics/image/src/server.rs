use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::quad,
        rendering::pbr_material_from_url,
        transform::{lookat_target, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(translation(), vec3(5., 5., 6.))
        .with(lookat_target(), vec3(0., 0., 2.))
        .with_default(main_scene())
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(
            pbr_material_from_url(),
            asset::url("assets/pipeline.json/0/mat.json").unwrap(),
        )
        .spawn();
}
