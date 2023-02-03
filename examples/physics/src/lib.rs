use components::core::{
    app::main_scene, camera::{
        active_camera, aspect_ratio, aspect_ratio_from_window, fovy, near, perspective_infinite_reverse, projection, projection_view
    }, ecs::dont_store, game_objects::player_camera, physics::{box_collider, dynamic, physics_controlled, plane_collider}, player::{player, user_id}, primitives::{cube, quad}, rendering::color, transform::{inv_local_to_world, local_to_world, lookat_center, lookat_up, rotation, scale, translation}
};
use tilt_runtime_scripting_interface::*;

#[main]
pub async fn main() -> EventResult {
    entity::game_object_base()
        .with_default(main_scene())
        .with(active_camera(), 0.)
        .with_default(dont_store())
        .with(translation(), vec3(5.0, 5.0, 4.0))
        .with_default(rotation())
        .with(lookat_up(), vec3(0., 0., 1.))
        .with(lookat_center(), vec3(0., 0., 0.))
        .with_default(local_to_world())
        .with_default(inv_local_to_world())
        .with(near(), 0.1)
        .with(fovy(), 1.0)
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio(), 1.)
        .with(aspect_ratio_from_window(), ())
        .with_default(projection())
        .with_default(projection_view())
        .spawn(false);

    entity::game_object_base()
        .with_default(cube())
        .with(box_collider(), vec3(2., 2., 2.))
        .with(dynamic(), true)
        .with_default(physics_controlled())
        .with(translation(), vec3(0., 0., 10.))
        .spawn(false);

    entity::game_object_base()
        .with_default(quad())
        .with_default(plane_collider())
        .with(dynamic(), false)
        .with_default(physics_controlled())
        .with(scale(), vec3(5., 5., 5.))
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn(false);

    EventOk
}
