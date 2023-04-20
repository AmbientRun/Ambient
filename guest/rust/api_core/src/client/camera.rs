use crate::{
    global::{EntityId, Ray, Vec2, Vec3},
    internal::{
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
};

/// Converts clip-space coordinates to a [Ray] in world space.
///
/// To obtain clip-space coordinates, use [screen_to_clip_space].
pub fn clip_space_ray(camera: EntityId, clip_space_position: Vec2) -> Ray {
    wit::client_camera::clip_space_ray(camera.into_bindgen(), clip_space_position.into_bindgen())
        .from_bindgen()
}

/// Converts a screen position (e.g. mouse position) to clip-space coordinates for the window.
pub fn screen_to_clip_space(screen_position: Vec2) -> Vec2 {
    wit::client_camera::screen_to_clip_space(screen_position.into_bindgen()).from_bindgen()
}

/// Converts a screen position (e.g. mouse position) to a [Ray] in world space.
pub fn screen_to_world_direction(camera: EntityId, screen_position: Vec2) -> Ray {
    wit::client_camera::screen_to_world_direction(camera.into_bindgen(), screen_position.into_bindgen()).from_bindgen()
}

/// Converts a world-space position to a screen position (e.g. mouse position).
pub fn world_to_screen(camera: EntityId, world_position: Vec3) -> Vec2 {
    wit::client_camera::world_to_screen(camera.into_bindgen(), world_position.into_bindgen()).from_bindgen()
}