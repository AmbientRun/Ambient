use components::core::{
    camera::{aspect_ratio, aspect_ratio_from_window, fovy, near, perspective_infinite_reverse, projection, projection_view},
    ecs::dont_store,
    game_objects::player_camera,
    player::{player, user_id},
    transform::{inv_local_to_world, local_to_world, rotation, translation},
};
use tilt_runtime_scripting_interface::*;

pub mod components;
pub mod params;

#[main]
pub async fn main() -> EventResult {
    // When a player spawns, give them a camera.
    spawn_query((player(), user_id())).bind(|players| {
        for (_, (_, player_user_id)) in players {
            entity::game_object_base()
                .with(user_id(), player_user_id)
                .with_default(player_camera())
                .with_default(dont_store())
                .with(translation(), vec3(0.0, 5.0, 0.0))
                .with(rotation(), Quat::from_rotation_x(90.0f32.to_radians()))
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
        }
    });

    // When a player despawns, delete their camera.
    let camera_query = query((player_camera(), user_id())).build();
    despawn_query((player(), user_id())).bind(move |players| {
        let player_cameras = camera_query.evaluate();
        for (_, (_, user_id)) in players {
            if let Some((id, _)) = player_cameras.iter().find(|(_, (_, camera_user_id))| user_id == *camera_user_id) {
                entity::despawn(*id);
            }
        }
    });

    let cube_ref = ObjectRef::new("assets/Cube.glb/objects/main.json");
    let cube_uid = entity::spawn_template(&cube_ref, Vec3::new(0.0, 0.0, 1.0), None, None, false);
    let cube_entity = entity::wait_for_spawn(&cube_uid).await;

    on(event::FRAME, move |_| {
        entity::set_rotation(cube_entity, Quat::from_axis_angle(Vec3::X, time().sin()));

        EventOk
    });

    EventOk
}
