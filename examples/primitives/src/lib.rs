use components::core::{
    app::main_scene,
    camera::{
        active_camera, aspect_ratio, aspect_ratio_from_window, fovy, near, perspective_infinite_reverse, projection, projection_view,
    },
    ecs::dont_store,
    primitives::{cube, quad},
    rendering::color,
    transform::{inv_local_to_world, local_to_world, lookat_center, lookat_up, rotation, scale, translation},
};
use tilt_runtime_scripting_interface::*;

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

    entity::game_object_base().with_default(cube()).with(translation(), vec3(0., 0., 1.)).spawn(false);
    entity::game_object_base().with_default(quad()).with(scale(), vec3(5., 5., 5.)).with(color(), vec4(1., 0., 0., 1.)).spawn(false);

    EventOk
}
