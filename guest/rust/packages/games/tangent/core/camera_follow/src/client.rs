use std::f32::consts::PI;

use ambient_api::{
    core::{
        camera::{
            components::{active_camera, fog, fovy},
            concepts::{
                PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
            },
        },
        messages::Frame,
        physics::components::linear_velocity,
        transform::components::{lookat_target, lookat_up, rotation, translation},
    },
    prelude::*,
};
use packages::tangent_schema::player::components as pc;

const CAMERA_OFFSET: Vec3 = vec3(1.5, 5.4, 1.8);

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
        let vehicle_ref = entity::get_component(player_id, pc::vehicle_ref());
        let target = vehicle_ref
            .and_then(vehicle_target)
            .unwrap_or_else(arena_target);

        entity::set_component(camera_id, translation(), target.position);
        entity::set_component(camera_id, lookat_target(), target.lookat);
        entity::set_component(camera_id, lookat_up(), Vec3::Z);
        entity::set_component(
            camera_id,
            fovy(),
            0.9 + (target.speed.abs() / 300.0).clamp(0.0, 1.0),
        );
        entity::set_component(
            camera_id,
            active_camera(),
            if vehicle_ref.is_none() { -1.0 } else { 2.0 },
        )
    });
}

struct CameraTarget {
    position: Vec3,
    lookat: Vec3,
    speed: f32,
}

fn vehicle_target(vehicle_id: EntityId) -> Option<CameraTarget> {
    let position = entity::get_component(vehicle_id, translation())?;
    let rotation = entity::get_component(vehicle_id, rotation())?;

    let target_position = position + rotation * CAMERA_OFFSET;
    Some(CameraTarget {
        position: target_position,
        lookat: target_position + rotation * -Vec3::Y,
        speed: entity::get_component(vehicle_id, linear_velocity())
            .unwrap_or_default()
            .length(),
    })
}

fn arena_target() -> CameraTarget {
    let time = game_time().as_secs_f32();
    let period = 20.0;

    let radius = 100.0;
    let height = 80.0;

    let position = vec3(
        radius * (time * PI / period).cos(),
        radius * (time * PI / period).sin(),
        height,
    );

    CameraTarget {
        position,
        lookat: Vec3::ZERO,
        speed: 0.0,
    }
}
