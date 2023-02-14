use kiwi_api::{
    components::core::{
        app::main_scene,
        camera::{active_camera, aspect_ratio_from_window, perspective_infinite_reverse},
        object::object_from_url,
        transform::{lookat_center, rotation, translation},
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

    let cube_id = entity::game_object_base()
        .with(object_from_url(), "assets/Cube.glb".to_string())
        .with(components::is_the_best(), true)
        .spawn(false);

    on(event::FRAME, move |_| {
        entity::set_component(
            cube_id,
            rotation(),
            Quat::from_axis_angle(Vec3::X, time().sin()),
        );

        EventOk
    });

    EventOk
}
