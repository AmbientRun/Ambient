use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        prefab::{prefab_from_url, spawned},
        transform::{lookat_center, rotation, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

#[main]
pub async fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(5., 5., 4.))
        .with(lookat_center(), vec3(0., 0., 0.))
        .spawn();

    let cube_id = Entity::new()
        .with_merge(make_transformable())
        .with(prefab_from_url(), asset::url("assets/Cube.glb").unwrap())
        .with(components::is_the_best(), true)
        .spawn();
    entity::wait_for_component(cube_id, spawned()).await;

    ambient_api::messages::Frame::subscribe(move |_| {
        entity::set_component(
            cube_id,
            rotation(),
            Quat::from_axis_angle(Vec3::X, time().sin()),
        );
    });
}
