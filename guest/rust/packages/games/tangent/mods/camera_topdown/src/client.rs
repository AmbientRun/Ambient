use ambient_api::{
    core::{
        camera::{
            components::{active_camera, fog},
            concepts::{
                PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
            },
        },
        messages::Frame,
        transform::components::{lookat_target, lookat_up, rotation, translation},
    },
    prelude::*,
};
use packages::tangent_schema::{player::components as pc, vehicle::client::components::speed_kph};

#[main]
pub fn main() {
    let camera_id = PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            translation: Some(vec3(0., 0., 20.)),
            main_scene: Some(()),
            aspect_ratio_from_window: Some(entity::resources()),
            ..default()
        },
        active_camera: -1.0,
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(fog(), ())
    .with(lookat_target(), vec3(0., 0., 0.))
    .with(lookat_up(), -Vec3::Y)
    .spawn();

    Frame::subscribe(move |_| {
        let player_id = player::get_local();
        let Some(vehicle_id) = entity::get_component(player_id, pc::vehicle_ref()) else {
            entity::set_component(camera_id, active_camera(), -1.0);
            return;
        };
        let Some(vehicle_position) = entity::get_component(vehicle_id, translation()) else {
            return;
        };
        let Some(vehicle_rotation) = entity::get_component(vehicle_id, rotation()) else {
            return;
        };
        let Some(vehicle_speed_kph) = entity::get_component(vehicle_id, speed_kph()) else {
            return;
        };

        let vehicle_yaw = vehicle_rotation.to_euler(glam::EulerRot::ZYX).0;
        let vehicle_yaw_rot = Quat::from_rotation_z(vehicle_yaw);

        // offset to put the vehicle more towards the bottom of the screen
        let base = vehicle_position + vehicle_yaw_rot * vec3(0., -7.5, 0.);
        let speed_factor = 0.9 + (vehicle_speed_kph.abs() / 300.0).clamp(0.0, 1.0);

        entity::set_component(
            camera_id,
            translation(),
            base + 20.0 * speed_factor * Vec3::Z,
        );
        entity::set_component(camera_id, lookat_target(), base);
        entity::set_component(camera_id, lookat_up(), vehicle_yaw_rot * -Vec3::Y);
        entity::set_component(camera_id, active_camera(), 5.0);
    });
}
