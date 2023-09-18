use ambient_api::{
    core::{
        camera::{
            components::{fog, fovy},
            concepts::{
                PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
            },
        },
        messages::Frame,
        transform::components::{lookat_target, rotation, translation},
    },
    prelude::*,
};
use packages::tangent_schema::{player::components as pc, vehicle::client::components as vcc};

const CAMERA_OFFSET: Vec3 = vec3(0.5, 1.8, 0.6);

#[main]
pub fn main() {
    let camera_id = PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            translation: Some(vec3(5., 5., 2.)),
            main_scene: Some(()),
            aspect_ratio_from_window: Some(entity::resources()),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(fog(), ())
    .with(lookat_target(), vec3(0., 0., 1.))
    .spawn();

    Frame::subscribe(move |_| {
        let player_id = player::get_local();
        let Some(vehicle_id) = entity::get_component(player_id, pc::vehicle_ref()) else {
            return;
        };
        let Some(vehicle_position) = entity::get_component(vehicle_id, translation()) else {
            return;
        };
        let Some(vehicle_rotation) = entity::get_component(vehicle_id, rotation()) else {
            return;
        };
        let Some(vehicle_speed_kph) = entity::get_component(vehicle_id, vcc::speed_kph()) else {
            return;
        };
        let Some(camera_position) = entity::get_component(camera_id, translation()) else {
            return;
        };
        let Some(camera_lookat) = entity::get_component(camera_id, lookat_target()) else {
            return;
        };

        // Smooth out the camera movement by moving towards the target with a constant velocity
        let target_position = vehicle_position + vehicle_rotation * CAMERA_OFFSET;
        let target_lookat = target_position + vehicle_rotation * -Vec3::Y;

        let dt = delta_time();
        const CAMERA_SPEED_MS: f32 = 15.0;
        const CAMERA_SNAP_TIME: f32 = 0.1;

        let camera_snap_distance_sqr = (CAMERA_SPEED_MS * CAMERA_SNAP_TIME).powi(2);

        // If we're almost at the target, just snap to it
        let s = if target_position.distance_squared(camera_position) < camera_snap_distance_sqr {
            1.0
        } else {
            CAMERA_SPEED_MS * dt
        };

        let new_position = camera_position.lerp(target_position, s);
        let new_lookat = camera_lookat.lerp(target_lookat, s);

        entity::set_component(camera_id, translation(), new_position);
        entity::set_component(camera_id, lookat_target(), new_lookat);
        entity::set_component(
            camera_id,
            fovy(),
            0.9 + (vehicle_speed_kph.abs() / 300.0).clamp(0.0, 1.0),
        );
    });
}
