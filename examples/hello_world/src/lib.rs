use components::core::{
    camera::{aspect_ratio, aspect_ratio_from_window, fovy, near, perspective_infinite_reverse, projection, projection_view},
    ecs::dont_store,
    game_objects::player_camera,
    player::{player, user_id},
    transform::{inv_local_to_world, local_to_world, rotation, translation},
};
use elements_runtime_scripting_interface::{player::KeyCode, *};

struct CameraState {
    radius: f32,
    yaw: f32,
}
impl Default for CameraState {
    fn default() -> Self {
        CameraState { radius: 5.0, yaw: 0.0 }
    }
}

#[main]
pub async fn main() -> EventResult {
    let camera_state = State::new(CameraState::default());

    // When a player spawns, give them a camera.
    spawn_query((player(), user_id())).bind({
        let camera_state = camera_state.clone();
        move |players| {
            for (_, (_, player_user_id)) in players {
                entity::game_object_base()
                    .with(user_id(), player_user_id)
                    .with_default(player_camera())
                    .with_default(dont_store())
                    .with(translation(), vec3(0.0, camera_state.borrow().radius, 0.0))
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

    camera_query.bind({
        let camera_state = camera_state.clone();
        move |cameras| {
            for (id, _) in cameras {
                let camera_state = camera_state.borrow();
                let yaw = camera_state.yaw;

                entity::set_position(id, Quat::from_rotation_z(yaw) * (Vec3::Y * camera_state.radius));
                entity::set_rotation(id, Quat::from_rotation_z(yaw) * Quat::from_rotation_x(90.0f32.to_radians()));
            }
        }
    });

    on(event::FRAME, move |_| {
        for player in player::get_all() {
            let Some((delta, new)) = player::get_raw_input_delta(player) else { continue; };
            let dt = frametime();

            let left = new.keys.contains(&KeyCode::A);
            let right = new.keys.contains(&KeyCode::D);
            let horizontal = ((left as u32 as f32) - (right as u32 as f32)) * dt;

            let mut camera_state = camera_state.borrow_mut();
            camera_state.radius = (camera_state.radius - (delta.mouse_wheel * dt)).max(2.0);
            camera_state.yaw += horizontal;
        }
        EventOk
    });

    loop {
        println!("Hello, world! It is {}", time());
        sleep(0.5).await;
    }
}
