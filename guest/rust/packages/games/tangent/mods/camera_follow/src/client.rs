use std::f32::consts::PI;

use ambient_api::{
    core::{
        camera::{
            components::{fog, fovy},
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

const CAMERA_OFFSET: Vec3 = vec3(0.5, 1.8, 0.6);

#[main]
pub fn main() {
    let camera_id = PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            translation: Some(vec3(0., 0., 20.)),
            main_scene: Some(()),
            aspect_ratio_from_window: Some(entity::resources()),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(fog(), ())
    .with(lookat_target(), vec3(0., 0., 0.))
    .with(lookat_up(), -Vec3::Y)
    .spawn();

    Frame::subscribe(move |_| {
        let Some(camera_position) = entity::get_component(camera_id, translation()) else {
            return;
        };
        let Some(camera_lookat) = entity::get_component(camera_id, lookat_target()) else {
            return;
        };

        let player_id = player::get_local();
        let target = entity::get_component(player_id, pc::vehicle_ref())
            .and_then(vehicle_target)
            .unwrap_or_else(arena_target);

        // Smooth out the camera movement by moving towards the target with a constant velocity
        let dt = delta_time();
        let camera_speed_ms = (target.speed * 5.0).max(2.0);
        const CAMERA_SNAP_TIME: f32 = 0.01;

        let camera_snap_distance_sqr = (camera_speed_ms * CAMERA_SNAP_TIME).powi(2);

        // If we're almost at the target, just snap to it
        let s = if target.position.distance_squared(camera_position) < camera_snap_distance_sqr {
            1.0
        } else {
            camera_speed_ms * dt
        };

        let new_position = camera_position.lerp(target.position, s);
        let new_lookat = camera_lookat.lerp(target.lookat, s);

        entity::set_component(camera_id, translation(), new_position);
        entity::set_component(camera_id, lookat_target(), new_lookat);
        entity::set_component(camera_id, lookat_up(), Vec3::Z);
        entity::set_component(
            camera_id,
            fovy(),
            0.9 + (target.speed.abs() / 300.0).clamp(0.0, 1.0),
        );
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
