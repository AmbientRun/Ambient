use crate::{
    global::{Vec2, Ray, EntityId},
    internal::{
        wit,
        conversion::{FromBindgen, IntoBindgen},
    },
};

/// Converts normalized mouse coordinate to Ray in world space 
#[cfg(feature = "client")]
pub fn screen_ray(camera: EntityId, screen_space_pos: Vec2) -> Ray {
    wit::camera::screen_ray(camera.into_bindgen(), screen_space_pos.into_bindgen()).from_bindgen()
}
