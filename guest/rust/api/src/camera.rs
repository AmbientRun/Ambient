use crate::{
    global::{EntityId, Ray, Vec2},
    internal::{
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
};

/// Converts normalized mouse coordinates to a [Ray] in world space.
pub fn screen_ray(camera: EntityId, screen_space_pos: Vec2) -> Ray {
    wit::client_camera::screen_ray(camera.into_bindgen(), screen_space_pos.into_bindgen())
        .from_bindgen()
}
