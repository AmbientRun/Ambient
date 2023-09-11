use ambient_api::{
    core::{
        camera::concepts::{
            PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
        },
        transform::components::{rotation, translation},
    },
    prelude::*,
};
use packages::this::components::{camera_angle, camera_distance, is_orbit_camera};

#[main]
pub fn main() {
    spawn_query(is_orbit_camera()).bind(|cameras| {
        for (camera_id, _) in cameras {
            entity::add_components(
                camera_id,
                PerspectiveInfiniteReverseCamera {
                    local_to_world: Mat4::IDENTITY,
                    near: 0.1,
                    projection: Mat4::IDENTITY,
                    projection_view: Mat4::IDENTITY,
                    active_camera: 0.0,
                    inv_local_to_world: Mat4::IDENTITY,
                    fovy: 1.0,
                    aspect_ratio: 1.0,
                    perspective_infinite_reverse: (),
                    optional: PerspectiveInfiniteReverseCameraOptional {
                        translation: Some(Vec3::ZERO),
                        main_scene: Some(()),
                        aspect_ratio_from_window: Some(entity::resources()),
                        ..default()
                    },
                },
            );
            entity::add_component_if_required(
                camera_id,
                camera_angle(),
                vec2(0.0, 45f32.to_radians()),
            );
            entity::add_component_if_required(camera_id, camera_distance(), 5.0);
        }
    });

    query(is_orbit_camera()).each_frame(|cameras| {
        let (delta, input) = input::get_delta();

        let angle_delta = if input.mouse_buttons.contains(&MouseButton::Right) {
            delta.mouse_position * 0.01
        } else {
            Vec2::ZERO
        };

        let distance_delta = delta.mouse_wheel * -0.1;

        for (camera_id, _) in cameras {
            let angle = entity::mutate_component_with_default(
                camera_id,
                camera_angle(),
                Vec2::ZERO,
                |rot| {
                    *rot += angle_delta;
                    rot.y = rot.y.clamp(-89f32.to_radians(), 89f32.to_radians());
                },
            );

            let distance =
                entity::mutate_component_with_default(camera_id, camera_distance(), 0.0, |dist| {
                    *dist = f32::max(*dist + distance_delta, 1.0);
                });

            let quat = Quat::from_euler(glam::EulerRot::ZXY, angle.x, -angle.y, 0.0);
            entity::add_component(camera_id, translation(), quat * vec3(0.0, -distance, 0.0));
            entity::add_component(camera_id, rotation(), quat);
        }
    });
}
