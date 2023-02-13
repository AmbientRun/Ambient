use kiwi_api::{
    components::core::{
        app::main_scene,
        camera::{active_camera, aspect_ratio_from_window, perspective_infinite_reverse},
        object::object_from_url,
        physics::{box_collider, dynamic, physics_controlled},
        primitives::cube,
        transform::{lookat_center, scale, translation},
    },
    prelude::*,
};

#[main]
pub async fn main() -> EventResult {
    entity::game_object_base()
        .with_default(main_scene())
        .with(active_camera(), 0.)
        .with(translation(), vec3(5.0, 5.0, 4.0))
        .with(lookat_center(), vec3(0., 0., 0.))
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio_from_window(), ())
        .spawn(false);

    entity::game_object_base()
        .with_default(cube())
        .with(box_collider(), vec3(2., 2., 2.))
        .with(dynamic(), true)
        .with_default(physics_controlled())
        .with(translation(), vec3(0., 0., 5.))
        .with(scale(), vec3(0.5, 0.5, 0.5))
        .spawn(false);

    entity::game_object_base()
        .with(object_from_url(), "assets/Shape.glb".to_string())
        .spawn(false);

    EventOk
}
