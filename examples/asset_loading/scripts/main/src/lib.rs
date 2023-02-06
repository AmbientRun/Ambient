use components::core::{
    app::main_scene, camera::{
        active_camera, aspect_ratio, aspect_ratio_from_window, fovy, near, perspective_infinite_reverse, projection, projection_view
    }, ecs::dont_store, game_objects::player_camera, player::{player, user_id}, transform::{inv_local_to_world, local_to_world, rotation, translation}
};
use tilt_runtime_scripting_interface::*;

pub mod components;
pub mod params;

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

    let cube_ref = ObjectRef::new("assets/Cube.glb/objects/main.json");
    let cube_uid = entity::spawn_template(&cube_ref, Vec3::new(0.0, 0.0, 1.0), None, None, false);
    let cube_entity = entity::wait_for_spawn(&cube_uid).await;

    on(event::FRAME, move |_| {
        entity::set_rotation(cube_entity, Quat::from_axis_angle(Vec3::X, time().sin()));

        EventOk
    });

    EventOk
}
