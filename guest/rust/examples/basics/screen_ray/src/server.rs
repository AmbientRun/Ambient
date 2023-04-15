use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::{quad, cube},
        transform::{lookat_center, translation},
        physics::plane_collider,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    physics::{raycast_first},
    prelude::*,
};

#[main]
pub async fn main() -> ResultEmpty {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with_default(plane_collider())
        .spawn();

    let cube_id = Entity::new()
        .with_merge(make_transformable())
        .with_default(cube())
        .spawn();

    messages::Input::subscribe(move |source, msg| {
        if let Some(hit) = raycast_first(msg.ray_origin, msg.ray_dir.normalize()) {
            // Set position of cube to the raycast hit position
            entity::set_component(cube_id, translation(), hit.position);
        }
    });

    OkEmpty
}
