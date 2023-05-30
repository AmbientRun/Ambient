use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        prefab::prefab_from_url,
        transform::{lookat_target, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(2.0, 2.0, 4.0))
        .with(lookat_target(), vec3(0.0, 0.0, 0.0))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with(translation(), vec3(-1.25, 0.0, 0.0))
        .with(
            prefab_from_url(),
            asset::url("assets/quad-linear.glb").unwrap(),
        )
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with(translation(), vec3(1.25, 0.0, 0.0))
        .with(
            prefab_from_url(),
            asset::url("assets/quad-nearest.glb").unwrap(),
        )
        .spawn();
}
