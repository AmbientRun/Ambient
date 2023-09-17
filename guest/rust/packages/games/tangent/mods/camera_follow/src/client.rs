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

        let camera_position = vehicle_position + vehicle_rotation * CAMERA_OFFSET;
        entity::set_component(camera_id, translation(), camera_position);
        entity::set_component(
            camera_id,
            lookat_target(),
            camera_position + vehicle_rotation * -Vec3::Y,
        );
        entity::set_component(
            camera_id,
            fovy(),
            0.9 + (vehicle_speed_kph.abs() / 300.0).clamp(0.0, 1.0),
        );
    });
}
